use std::fmt::Display;

#[macro_export]
macro_rules! try_or_redirect {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                use utils::route::failure_redirect;
                failure_redirect(e);
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! try_or_redirect_opt {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                use utils::route::failure_redirect;
                failure_redirect(e);
                return None;
            }
        }
    };
}

pub fn failure_redirect<E: Display>(err: E) {
    let path = format!("/error?err={err}");
    #[cfg(feature = "hydrate")]
    {
        let nav = leptos_router::hooks::use_navigate();
        nav(&path, Default::default());
    }
    #[cfg(not(feature = "hydrate"))]
    {
        use leptos_axum::redirect;
        redirect(&path);
    }
}

pub fn go_to_root() {
    let path = "/";
    #[cfg(feature = "hydrate")]
    {
        let nav = leptos_router::hooks::use_navigate();
        nav(path, Default::default());
    }
    #[cfg(not(feature = "hydrate"))]
    {
        use leptos_axum::redirect;
        redirect(&path);
    }
}
