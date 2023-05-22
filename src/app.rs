use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
	async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
	provide_meta_context(cx);

	view! {
		cx,
		<Stylesheet id="leptos" href="/pkg/tailwind.css"/>
		<Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
		<Router>
			<Routes>
				<Route path="" view=  move |cx| view! { cx, <Home/> }/>
			</Routes>
		</Router>
	}
}

#[component]
fn Home(cx: Scope) -> impl IntoView {
	let (count, set_count) = create_signal(cx, 0);

	view! { cx,
		<main class="my-0 mx-auto max-w-3xl text-center">
			<h2 class="p-6 text-4xl text-rp-text">"Welcome to Leptos with Tailwind"</h2>
			<p class="px-10 pb-10 text-left text-rp-text">"Tailwind will scan your Rust files for Tailwind class names and compile them into a CSS file."</p>
			<button
				class="inline-flex h-12 items-center rounded-2xl bg-muted/10 px-6 font-medium transition hover:bg-muted/20"
				on:click=move |_| set_count.update(|count| *count += 1)
			>
				"Something's here | "
				{move || if count.get() == 0 {
					"Click me!".to_string()
				} else {
					count.get().to_string()
				}}
				" | Some more text"
			</button>
		</main>
	}
}
