use super::UnifiedDispatcher;
use specs::prelude::*;
use crate::effects;

pub struct MultiThreadedDispatcher {
    pub dispatcher: Dispatcher<'static, 'static>
}

impl<'a> UnifiedDispatcher for MultiThreadedDispatcher {
    fn run_now(&mut self, ecs: *mut World) {
        unsafe {
            self.dispatcher.dispatch(&mut *ecs);
            effects::run_effects_queue(&mut *ecs);
        }
    }
}

macro_rules! construct_dispatcher {
    (
        $(
            (
                $type:ident,
                $name:expr,
                $deps:expr
            )
        ),*
    ) => {
        fn new_dispatch() -> Box<dyn UnifiedDispatcher + 'static> {
            use specs::DispatcherBuilder;

            let dispatcher = DispatcherBuilder::new()
                $(
                    .with($type{}, $name, $deps)
                )*
                .build();

            let dispatch = MultiThreadedDispatcher{ dispatcher };

            Box::new(dispatch)
        }
    };
}
