use std::future::Future;

#[cfg(target_arch = "wasm32")]
pub fn create_task<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_task<F>(future: F)
where
    F: Future<Output = ()> + 'static + Send,
{
    tokio::spawn(future);
}
