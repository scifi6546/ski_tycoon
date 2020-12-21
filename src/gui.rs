use generational_arena::{Arena, Index as ArenaIndex};
use std::collections::HashMap;
use nalgebra::Vector2;
/// Temporary  Object representing gui rendering
#[derive(Clone,Debug)]
struct RenderObject {}
struct GuiContainer {
    elements: Arena<Box<dyn GuiElement>>,
    /// points to index of parent in world arena
    parent_index: ArenaIndex,
}
struct RenderGuiContainer{
    parent: RenderObject,
    children: Vec<RenderObject>,
}
impl GuiContainer {
    fn get_screen_collider(&self) -> Vec<Triangle> {
        todo!()
    }
    fn get_render_model(&self)->&RenderGuiContainer{
        todo!()
    }
    fn update_render_gui_container(&mut self){
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
#[derive(Clone,Debug)]
enum Message {
    /// Clicked on Mesh
    ClickedOn
}
enum StateChange {
    NoChange,
    UpdateGui,
    DeleteParent
}
trait GuiElement {
    fn get_box(&self) -> BoundingBox;
    /// Recieves events from the runtime. Includes thing like click events. If the state is changed get model will be called.
    fn process_event(&self, event: Event) -> (StateChange, Vec<Message>);
    /// Gets the model. Should only be called when first constructed or process event returns `StateChange::UpdateGui`
    fn get_model(&self) -> RenderObject;
    /// Gets collider triangle in screen coordinates
    fn get_screen_collider(&self) -> Vec<Triangle>;
}
enum GetGuiOutput {
    //Spawn a container. If one already exists replace existing gui with current container
    Contianer(GuiContainer),
    //do not change Container
    NoChange,
    //No gui to be emmitted. If one exists delete the gui
    None,
}

struct Triangle {
    points: [Vector2<f32>; 3],
}
impl Triangle {
    pub fn intersects(&self, point: &Vector2<f32>) -> bool {
        todo!("figure out triangle does intersect")
    }
}
//represents an object that may own a container
trait GuiParent {
    fn get_gui(&self) -> GetGuiOutput;
    fn process_message(&mut self, message: &Message);
    /// Gets collider triangle in screen coordinates
    fn get_screen_collider(&self) -> Vec<Triangle>;
}
struct GuiState {
    containers: HashMap<ArenaIndex,GuiContainer>,
}
impl GuiState {
    #[allow(dead_code)]
    pub fn game_loop(
        &mut self,
        events: EventPacket,
        objects: &mut Arena<Box<dyn GuiParent>>,
    ) -> Vec<RenderObject> {
        //1. process event. Mark key if state needs changing
        let mut update_gui = vec![];
        let mut messages = vec![];
        for event in events.events.iter() {
            let (mut gui,mut msg) = match event{
                Event::MouseEvent(m)=>self.process_mouse_gui(m)
            };
            update_gui.append(&mut gui);
            messages.append(&mut msg);
        }
        //2. Update gui boxes with specific keys marked by update
        for (state,parent_index,child_index) in update_gui.iter(){
            match state{
                StateChange::NoChange=>(),
                StateChange::UpdateGui=>{
                    if let Some(parent) = self.containers.get_mut(parent_index){
                        if let Some(child) = parent.elements.get(*child_index){
                            parent.update_render_gui_container();
                        }
                    }
                },
                StateChange::DeleteParent=>
                    if self.containers.contains_key(parent_index){
                        self.containers.remove(parent_index);
                    }
                
            }
        }
        //3. send on click
        for (index, object) in objects.iter(){
            for triangle in object.get_screen_collider(){
                if triangle.intersects(&events.mouse_position){
                    messages.push((index,Message::ClickedOn));
                }
            }
        }
        //4. send messages to owning objects
        for (index,message) in messages.iter(){
            objects[*index].process_message(message);
        }
        //5. Get gui from all items. update gui as nessecary
        for (index,object) in objects.iter(){
            match object.get_gui(){
                GetGuiOutput::Contianer(c)=>{self.containers.insert(index,c);},
                GetGuiOutput::NoChange=>(),
                GetGuiOutput::None=>{self.containers.remove(&index);},
            }
        }
        //6. Render Gui
        let mut output = vec![];
        for container in self.containers.iter().map(|(_i,c)|c.get_render_model()){
            output.push(container.parent.clone());
            output.append(&mut container.children.clone());

        }
        return output;
    }
    /// checks if mouse interesected with one part of the gui. First return argument is the list of guis elements 
    /// that need updating. Second is a vector of (Index of Gameobjects to Send message to, Message to send)
    fn process_mouse_gui(
        &mut self,
        mouse: &MouseEvent,
    ) -> (Vec<(StateChange,ArenaIndex, ArenaIndex)>, Vec<(ArenaIndex, Message)>) {
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
                        let (state_change,messages) = element.process_event(Event::MouseEvent(mouse.clone()));
                        update_events.push((state_change,parent_index.clone(), child_index));
                        for msg in messages.iter(){
                            update_mesages.push((container.parent_index,msg.clone()));
                        }
                    }
                }
            }
        }
        return (update_events,update_mesages);
    }
}
