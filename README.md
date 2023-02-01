# egui-opengl-internal
Rust OpenGL backend for [egui](https://github.com/emilk/egui) using opengl internally to render egui while injected into a game. Thank you to the combination of [egui-d3d11](https://github.com/sy1ntexx/egui-d3d11) and [egui-d3d9](https://github.com/unknowntrojan/egui-d3d9) for the app structure/hooking code and [egui_glfw_gl](https://github.com/cohaereo/egui_glfw_gl) for the shader and painter.

It uses [GetProcAddress](https://docs.rs/winapi/latest/winapi/um/libloaderapi/fn.GetProcAddress.html) and [wglGetProcAddress](https://docs.rs/winapi/latest/winapi/um/wingdi/fn.wglGetProcAddress.html) to load [gl-rs](https://github.com/brendanzab/gl-rs/) functions with [gl::load_with](https://docs.rs/gl/0.14.0/gl/fn.load_with.html).

#
![](media/demo.gif)
