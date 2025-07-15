#[macro_export]
macro_rules! icon_gen {
    ($name:ident, view_box=$view_box:expr, $path:expr) => {
        #[allow(non_upper_case_globals)]
        pub static $name: icondata::Icon = &icondata_core::IconData {
            style: None,
            x: None,
            y: None,
            width: None,
            height: None,
            view_box: Some($view_box),
            stroke_linecap: None,
            stroke_linejoin: None,
            stroke_width: None,
            stroke: None,
            fill: None,
            data: $path,
        };
    };
}

// pub(crate) use icon_gen;
