use crate::registered_component_types;
use hecs::{ColumnBatchType, Entity, World};
use serde::{Deserialize, Serialize, de::Error};

/// stores a hot-reload-persistent snapshot of a [hecs::World] that preserves field names etc. by
/// storing in JSON
#[derive(Serialize, Deserialize)]
pub struct Snapshot {
	pub archetypes: Vec<ArchetypeRecord>,
}

/// stores all entities of that archetype and the columns (<=> components) for each entity
#[derive(Serialize, Deserialize)]
pub struct ArchetypeRecord {
	pub entities: Vec<u64>,
	pub columns: Vec<ComponentColumnRecord>,
}

/// stores the component type name as well as a JSON serialized [hecs::ArchetypeColumn]
#[derive(Serialize, Deserialize)]
pub struct ComponentColumnRecord {
	pub component_type_name: String,
	/// serialized to a json string
	/// (this is kind of the most performant way for storing the components fields across
	/// hot-reloads, since storing a serde_json::Value instead has much more memory overhead for
	/// each component)
	pub serialized_components: String,
}

/// serializes a [hecs::World] to a [Snapshot]
pub fn serialize(world: &World) -> serde_json::Result<Snapshot> {
	let archetypes = world
		.archetypes()
		.filter(|archetype| !archetype.is_empty())
		.map(|archetype| {
			let entities = archetype
				.ids()
				.iter()
				.map(|&id| unsafe { world.find_entity_from_id(id) }.to_bits().get())
				.collect();

			let column_records = registered_component_types()
				.into_iter()
				.filter(|component_type| (component_type.has)(archetype))
				.map(|component_type| {
					Ok(ComponentColumnRecord {
						component_type_name: (component_type.type_name)().to_string(),
						serialized_components: (component_type.serialize)(archetype)?,
					})
				})
				.collect::<serde_json::Result<Vec<_>>>()?;

			Ok(ArchetypeRecord {
				entities,
				columns: column_records,
			})
		})
		.collect::<serde_json::Result<Vec<_>>>()?;

	Ok(Snapshot { archetypes })
}

/// deserializes a [Snapshot] back to a [hecs::World]
pub fn deserialize(snapshot: Snapshot) -> serde_json::Result<World> {
	let mut world = World::new();

	for archetype in snapshot.archetypes {
		let entities = archetype
			.entities
			.into_iter()
			.map(|bits| {
				Entity::from_bits(bits).ok_or(serde_json::Error::custom(
					"snapshot contains an invalid entity id",
				))
			})
			.collect::<serde_json::Result<Vec<_>>>()?;

		let mut batch_type = ColumnBatchType::new();
		let mut known = Vec::new();
		for column in archetype.columns {
			// skip components this build doesn't know about (hot-reload compat)
			let Some(component_type) = registered_component_types()
				.into_iter()
				.find(|component_type| (component_type.type_name)() == column.component_type_name)
			else {
				continue;
			};
			(component_type.add_to_batch_type)(&mut batch_type);
			known.push((component_type, column.serialized_components));
		}

		let mut batch_builder = batch_type.into_batch(entities.len() as u32);
		for (component_type, components) in known {
			(component_type.deserialize)(&components, &mut batch_builder)?;
		}

		let batch = batch_builder.build().map_err(|e| {
			serde_json::Error::custom(format!("column batch was left incomplete: {e}"))
		})?;

		world.spawn_column_batch_at(&entities, batch);
	}

	Ok(world)
}
