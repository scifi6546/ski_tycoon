import * as rust from "rust";
let events = []
function canvas_click(){
    console.log("clicked??")
    document.getElementById("canvas").requestPointerLock();
}
function mouse_move(event){
    console.log(event)
    events.push({"name":"mouse_move","x":event.clientX,"y":event.clientY,"buttons":event.buttons})
}
document.getElementById("canvas").onclick=canvas_click;
document.getElementById("canvas").onmousemove = mouse_move
let  game = rust.init_game();
function render(){
    rust.render_frame(game,events);
    events=[]
    requestAnimationFrame(render)
}
requestAnimationFrame(render)