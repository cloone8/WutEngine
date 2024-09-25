use core::any::TypeId;

use crate::archetype::ArchetypeId;

use super::World;

impl World {
    #[track_caller]
    pub fn assert_coherent<const DEBUG_ONLY: bool>(&self) {
        if DEBUG_ONLY && !cfg!(debug_assertions) {
            return;
        }

        // Entity checks
        for (entity_id, containing_archetype_id) in &self.entities {
            if containing_archetype_id.is_none() {
                continue;
            }

            let containing_archetype = self
                .archetypes
                .get(containing_archetype_id.as_ref().unwrap());

            assert!(
                containing_archetype.is_some(),
                "Could not find containing archetype for entity {:?}",
                entity_id
            );

            let containing_archetype = containing_archetype.unwrap();
            containing_archetype.assert_coherent::<DEBUG_ONLY>();

            assert!(
                containing_archetype
                    .get_contained_entities()
                    .iter()
                    .any(|e| e == entity_id),
                "Containing archetype for entity {:?} does not actually contain the entity",
                entity_id
            );
        }

        // Archetype checks
        for (archetype_id, archetype) in &self.archetypes {
            let archetype_types: Vec<TypeId> = archetype.get_contained_types().copied().collect();
            let computed_archetype_id = ArchetypeId::new(&archetype_types);

            assert_eq!(*archetype_id, computed_archetype_id, "Computer archetype ID for archetype does not match its world key. Expected {:?}, computed {:?}", archetype_id, computed_archetype_id);
            assert!(!archetype.is_empty(), "Empty archetype found");

            // Now check if we can actually find ourselves in the type_containers map
            for archetype_type in archetype_types {
                let type_container =
                    self.type_containers
                        .get(&archetype_type)
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find type container for TypeId {:?}",
                                archetype_type
                            )
                        });

                assert!(
                    type_container.contains(archetype_id),
                    "ArchetypeId {:?} not found in TypeId container for type {:?}, even though it should have been there",
                    archetype_id,
                    archetype_type
                );
            }
        }

        // Type container checks
        for (type_id, type_id_containers) in &self.type_containers {
            for type_id_container_id in type_id_containers {
                let type_id_container = self.archetypes.get(type_id_container_id);

                assert!(
                    type_id_container.is_some(),
                    "Could not find container for TypeId {:?} and archetype id {:?}",
                    type_id,
                    type_id_container_id
                );

                let type_id_container = type_id_container.unwrap();

                assert!(
                    type_id_container
                        .get_contained_types()
                        .any(|e| e == type_id),
                    "Container for TypeId {:?} does not actually contain the TypeId",
                    type_id
                );
            }
        }
    }
}
