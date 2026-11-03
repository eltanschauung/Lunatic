fn hook(o:&ParticleOptions,p:[f64;3],v:[f64;3])->core::ops::ControlFlow<Option<Particle>> {
 match lunatic_particles::lampad_particle(o,p,v){Some(p)=>core::ops::ControlFlow::Break(Some(p)),None=>core::ops::ControlFlow::Continue(())}
}
