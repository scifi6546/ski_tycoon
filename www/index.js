import * as rust from "rust";
let last_x = null;
let last_y = null;
let last_time = new Date();
let events = []
function canvas_click(){
    console.log("clicked??")
    document.getElementById("canvas").requestPointerLock();
}
function mouse_move(event){
    console.log(event)
    let now = new Date();
    let mouse_event = new Map();
    if(last_x ===null){
        last_x = event.clientX;


    }
    if(last_y === null){
        last_y = event.clientY;
    }
    let delta_x = Number(event.clientX - last_x);
    console.log(delta_x)
    mouse_event.set("name","mouse_move");
    mouse_event.set("delta_x",event.clientX - last_x);
    mouse_event.set("delta_y",event.clientY - last_y);
    mouse_event.set("delta_time_ms",Number(now-last_time))
    mouse_event.set("buttons",event.buttons);
    events.push(mouse_event)
    last_x = event.clientX;
    last_y = event.clientY;
    last_time = now;
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