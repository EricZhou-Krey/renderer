pub mod app;
pub mod asset;
pub mod command;
pub mod component;
pub mod renderer;
pub mod resource;

/*
ORDER OF OPERATIONS (many, MANY mini steps inbetween):

* 1.5 -> implement deletion if VRAM bottlenecks, maybe a smart pointer implementation when dropping MeshId, TextureId, MaterialId created by slotmap

2. Render audio rays, for debug purposes, display bouncing, speed, collsisions, reflection rays, transmitted rays and etc.
3. Hook onto audio and map ray collsisons and %ray coverage to the audio output.

*/
