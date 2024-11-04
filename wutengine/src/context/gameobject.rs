use crate::component::data::ComponentData;
use crate::component::Component;
use crate::gameobject::GameObject;

/// The context for interacting with the current [GameObject]. Usually within a component
#[must_use = "The commands within the context must be consumed"]
#[derive(Debug)]
pub struct GameObjectContext<'a> {
    /// The reference to the [GameObject] itself.
    pub object: &'a GameObject,
    component_chunks: Vec<&'a mut [ComponentData]>,
    new_components: Vec<Box<dyn Component>>,
}

impl<'a> GameObjectContext<'a> {
    /// Creates a new [GameObjectContext] for the given [GameObject] and
    /// the component at the given index.
    pub(crate) fn new(
        gameobject: &'a GameObject,
        components: &'a mut [ComponentData],
        component_idx: usize,
    ) -> (&'a mut ComponentData, Self) {
        assert!(component_idx < components.len(), "Component out of range");

        let (before, rest) = components.split_at_mut(component_idx);
        let (component, after) = rest.split_at_mut(1);

        assert_eq!(1, component.len());

        let component = &mut component[0];

        let go_contex = GameObjectContext {
            object: gameobject,
            component_chunks: vec![before, after],
            new_components: Vec::new(),
        };

        (component, go_contex)
    }

    /// Returns the commands contained within the context
    pub(crate) fn consume(self) -> Vec<Box<dyn Component>> {
        self.new_components
    }

    /// Adds a new component to the [GameObject]
    pub fn add_component<T: Component>(&mut self, component: T) {
        self.new_components.push(Box::new(component));
    }

    /// Returns an immutable reference to the first component of type `T`, if the [GameObject] has any.
    pub fn get_component<T: Component>(&self) -> Option<&T> {
        for chunk in &self.component_chunks {
            for component_data in chunk.iter() {
                let as_ref = component_data.component.as_ref().as_any();
                let cast = as_ref.downcast_ref::<T>();

                if let Some(cast_ok) = cast {
                    return Some(cast_ok);
                }
            }
        }

        None
    }

    /// Returns immutable references to all components of type `T`, if the [GameObject] has any.
    pub fn get_components<T: Component>(&self) -> Vec<&T> {
        let mut found = Vec::new();

        for chunk in &self.component_chunks {
            for component_data in chunk.iter() {
                let as_ref = component_data.component.as_ref().as_any();
                let cast = as_ref.downcast_ref::<T>();

                if let Some(cast_ok) = cast {
                    found.push(cast_ok);
                }
            }
        }

        found
    }

    /// Returns a mutable reference to the first component of type `T`, if the [GameObject] has any.
    pub fn get_component_mut<T: Component>(&mut self) -> Option<&mut T> {
        for chunk in &mut self.component_chunks {
            for component_data in chunk.iter_mut() {
                let as_mut = component_data.component.as_mut().as_any_mut();
                let cast = as_mut.downcast_mut::<T>();

                if let Some(cast_ok) = cast {
                    return Some(cast_ok);
                }
            }
        }

        None
    }

    /// Returns mutable references to all components of type `T`, if the [GameObject] has any.
    pub fn get_components_mut<T: Component>(&mut self) -> Vec<&mut T> {
        let mut found = Vec::new();

        for chunk in &mut self.component_chunks {
            for component_data in chunk.iter_mut() {
                let as_mut = component_data.component.as_mut().as_any_mut();
                let cast = as_mut.downcast_mut::<T>();

                if let Some(cast_ok) = cast {
                    found.push(cast_ok);
                }
            }
        }

        found
    }
}
