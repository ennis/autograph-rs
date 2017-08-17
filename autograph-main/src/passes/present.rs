
/*gfx_pass!{
pass Present(frame: &'pass gfx::Frame, target: &'pass Arc<gfx::Framebuffer>)
{
    read {
        texture2D source {},
    }
    write {
    }
    pipeline BLIT_2D {
        path: "data/shaders/blit.glsl"
    }
    execute
    {

    }
}

}*/