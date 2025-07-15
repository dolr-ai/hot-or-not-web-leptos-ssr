use leptos::prelude::*;

pub type ABComponent<T> = Box<dyn Fn() -> Option<View<T>>>;

#[macro_export]
macro_rules! abselector_id {
    ($identifier:expr, $component_1:expr, $($rest:expr),* $(,)?) => {
        {
            let identifier = $identifier.as_deref();
            let mut view = None;

            // Reverse iterate through the components and create the match cases
            $(
                if identifier == Some(stringify!($rest).trim_start_matches("component_")) {
                    view = Some($rest.into_view());
                }
            )*

            // Fallback to the first component if no match is found
            view.unwrap_or_else(|| $component_1.into_view())
        }
    };
}

/*
USAGE:
    earlier:

    view! {
        <Scroller />
    }

    now:

    view! {
        let component_1: ABComponent = Box::new(<Scroller1 />);
        let component_2: ABComponent = Box::new(<Scroller2 />);
        let component_3: ABComponent = Box::new(<Scroller3 />);
        abselector!(/* closure returning string */, component_1, component_2, component_3)()
    }
*/
#[macro_export]
macro_rules! abselector {
    // Accept a closure for identifier generation
    ($identifier_closure:expr, $component_1:expr, $($rest:expr),* $(,)?) => {
        {
            // Execute the closure to get the identifier
            let identifier_string = ($identifier_closure)();
            let identifier = identifier_string.as_deref();
            let mut view = None;

            // Reverse iterate through the components and create the match cases
            $(
                if identifier == Some(stringify!($rest).trim_start_matches("component_")) {
                    view = Some($rest);
                }
            )*

            // Fallback to the first component if no match is found
            view.unwrap_or_else(|| $component_1)
        }
    };
}

#[macro_export]
macro_rules! abselector_alt {
    // Accept a closure for identifier generation
    ($identifier_closure:expr, $component_1:expr, $($rest:expr),* $(,)?) => {
        {
            // Execute the closure to get the identifier
            let identifier_string = ($identifier_closure)();
            let identifier = identifier_string.as_deref();
            let mut view = None;

            // Reverse iterate through the components and create the match cases
            $(
                if identifier == Some(stringify!($rest).trim_start_matches("component_")) {
                    view = Some($rest.into_view());
                }
            )*

            // Fallback to the first component if no match is found
            view.unwrap_or_else(|| $component_1.into_view())
        }
    };
}
