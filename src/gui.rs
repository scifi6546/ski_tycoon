use super::prelude::Model;
use generational_arena::{Arena, Index as ArenaIndex};
use nalgebra::Vector2;
use std::collections::{HashMap, HashSet};
use std::marker::Sized;
struct GuiContainer<RenderModel> {
    elements: Arena<Box<dyn GuiElement<RenderModel>>>,
    render_models: Arena<Option<RenderModel>>,
    /// points to index of parent in world arena
    parent_index: ArenaIndex,
}
impl<RenderModel: Sized> GuiContainer<RenderModel> {
    fn get_screen_collider(&self) -> Vec<Triangle> {
        todo!()
    }
    fn get_render_model(&self) -> Vec<RenderModel> {
        todo!()
    }
    // to call after state updates
    fn get_model(&mut self) -> HashMap<ArenaIndex, Model> {
        todo!()
    }
    fn submit_model(&mut self, models: &HashMap<ArenaIndex, RenderModel>) {
        todo!()
    }
}
struct BoundingBox {}
struct EventPacket {
    mouse_position: Vector2<f32>,
    events: Vec<Event>,
}
#[derive(Clone)]
struct MouseEvent {
    mouse_position: Vector2<f32>,
}
enum KeyboardKey {}
enum Event {
    MouseEvent(MouseEvent),
}
//wheter or not to update gui
#[derive(Clone, Debug)]
pub enum Message {
    /// Clicked on Mesh
    ClickedOn,
}
enum StateChange {
    NoChange,
    UpdateGui,
    DeleteParent,
}
trait GuiElement<RenderModel> {
    fn get_box(&self) -> BoundingBox;
    /// Recieves events from the runtime. Includes thing like click events. If the state is changed get model will be called.
    fn process_event(&self, event: Event) -> (StateChange, Vec<Message>);
    /// Gets the model. Should only be called when first constructed or process event returns `StateChange::UpdateGui`
    fn get_model(&self) -> RenderModel;
    /// Gets collider triangle in screen coordinates
    fn get_screen_collider(&self) -> Vec<Triangle>;
}
pub enum GetGuiOutput<RenderModel> {
    //Spawn a container. If one already exists replace existing gui with current container
    Contianer(GuiContainer<RenderModel>),
    //do not change Container
    NoChange,
    //No gui to be emmitted. If one exists delete the gui
    None,
}

pub struct Triangle {
    points: [Vector2<f32>; 3],
}
impl Triangle {
    pub fn intersects(&self, point: &Vector2<f32>) -> bool {
        todo!("figure out triangle does intersect")
    }
}
//represents an object that may own a container
pub trait GuiParent<RenderModel> {
    fn get_gui(&self) -> GetGuiOutput<RenderModel>;
    fn process_message(&mut self, message: &Message);
    /// Gets collider triangle in screen coordinates
    fn get_screen_collider(&self) -> Vec<Triangle>;
}
struct GuiState<RenderModel: Clone> {
    containers: HashMap<ArenaIndex, GuiContainer<RenderModel>>,
}
/// What needs to get changed (tommorow)
/// Add two new functions that look like
/// ```
/// //processes events and sends new models to update
/// fn process_events(&mut self,events: EventPacket,objects: &mut Arena<Box<dyn GuiParent>>,)->HashMap<ArenaIndex,Model>;
/// //submitting hashmap
/// fn submit_model(&mut self,map: HashMap<(ArenaIndex,ArenaIndex),RuntimeModel>);
/// //called after process event every frame. gets all models to be drawn.
/// fn get_runtime_model(&self)->Vec<RuntimeModel>;
/// ```
impl<RenderModel: Clone> GuiState<RenderModel> {
    #[allow(dead_code)]
    pub fn game_loop(
        &mut self,
        events: EventPacket,
        objects: &mut Arena<Box<dyn GuiParent<RenderModel>>>,
    ) -> HashMap<(ArenaIndex, ArenaIndex), Model> {
        //list of models to update
        let mut to_update = HashSet::new();
        //1. process event. Mark key if state needs changing
        let mut update_gui = vec![];
        let mut messages = vec![];
        for event in events.events.iter() {
            let (mut gui, mut msg) = match event {
                Event::MouseEvent(m) => self.process_mouse_gui(m),
            };
            update_gui.append(&mut gui);
            messages.append(&mut msg);
        }

        //2. Update gui boxes with specific keys marked by update
        for (state, parent_index, child_index) in update_gui.iter() {
            match state {
                StateChange::NoChange => (),
                StateChange::UpdateGui => {
                    if let Some(parent) = self.containers.get_mut(parent_index) {
                        if let Some(child) = parent.elements.get(*child_index) {
                            to_update.insert(parent_index.clone());
                        }
                    }
                }
                StateChange::DeleteParent => {
                    if self.containers.contains_key(parent_index) {
                        self.containers.remove(parent_index);
                        to_update.insert(parent_index.clone());
                    }
                }
            }
        }
        //3. send on click
        for (index, object) in objects.iter() {
            for triangle in object.get_screen_collider() {
                if triangle.intersects(&events.mouse_position) {
                    messages.push((index, Message::ClickedOn));
                }
            }
        }
        //4. send messages to owning objects
        for (index, message) in messages.iter() {
            objects[*index].process_message(message);
        }
        //5. Get gui from all items. update gui as nessecary
        for (index, object) in objects.iter() {
            match object.get_gui() {
                GetGuiOutput::Contianer(c) => {
                    self.containers.insert(index, c);
                    to_update.insert(index.clone());
                }
                GetGuiOutput::NoChange => (),
                GetGuiOutput::None => {
                    self.containers.remove(&index);
                }
            }
        }
        let mut output = HashMap::<(ArenaIndex, ArenaIndex), Model>::new();
        //6. get deltas
        for index in to_update.iter() {
            if let Some(container) = self.containers.get_mut(index) {
                let models = container.get_model();
                for (child_idx, m) in models.iter() {
                    output.insert(((index.clone()).clone(), child_idx.clone()), m.clone());
                }
            }
        }
        return output;
    }
    pub fn submit_model(&mut self, map: HashMap<(ArenaIndex, ArenaIndex), RenderModel>) {
        let mut model_map: HashMap<ArenaIndex, HashMap<ArenaIndex, RenderModel>> = HashMap::new();
        for ((container_idx, child_idx), runtime_model) in map.iter() {
            if let Some(container) = model_map.get_mut(&container_idx.clone()) {
                container.insert(child_idx.clone(), runtime_model.clone());
            } else {
                model_map.insert(
                    container_idx.clone(),
                    HashMap::<ArenaIndex, RenderModel>::new(),
                );
                model_map
                    .get_mut(container_idx)
                    .unwrap()
                    .insert(child_idx.clone(), runtime_model.clone());
            }
        }
        for (container_idx, map) in model_map.iter() {
            self.containers
                .get_mut(container_idx)
                .unwrap()
                .submit_model(map);
        }
    }
    pub fn get_runtime_model(&self) -> Vec<RenderModel> {
        let mut out_vec = vec![];
        for (_idx, container) in self.containers.iter() {
            out_vec.append(&mut container.get_render_model());
        }
        return out_vec;
    }
    ///
    /// checks if mouse interesected with one part of the gui. First return argument is the list of guis elements
    /// that need updating. Second is a vector of (Index of Gameobjects to Send message to, Message to send)
    fn process_mouse_gui(
        &mut self,
        mouse: &MouseEvent,
    ) -> (
        Vec<(StateChange, ArenaIndex, ArenaIndex)>,
        Vec<(ArenaIndex, Message)>,
    ) {
        let mut update_events = vec![];
        let mut update_mesages = vec![];
        for (parent_index, container) in self.containers.iter_mut() {
            if container
                .get_screen_collider()
                .iter()
                .map(|t| t.intersects(&mouse.mouse_position))
                .fold(false, |acc, x| acc | x)
            {
                for (child_index, element) in container.elements.iter() {
                    if element
                        .get_screen_collider()
                        .iter()
                        .map(|t| t.intersects(&mouse.mouse_position))
                        .fold(false, |acc, x| acc | x)
                    {
                        let (state_change, messages) =
                            element.process_event(Event::MouseEvent(mouse.clone()));
                        update_events.push((state_change, parent_index.clone(), child_index));
                        for msg in messages.iter() {
                            update_mesages.push((container.parent_index, msg.clone()));
                        }
                    }
                }
            }
        }
        return (update_events, update_mesages);
    }
}
