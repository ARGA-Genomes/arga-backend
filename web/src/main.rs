use dioxus::prelude::*;
use serde::{Deserialize, Serialize};


const MAIN_CSS: Asset = asset!("/assets/main.css");


fn main() {
    dioxus::launch(App);
}


fn admin_api(path: &str) -> String {
    let origin = match web_sys::window() {
        Some(window) => window.origin(),
        None => "".to_string(),
    };
    format!("{origin}/api/admin/{path}")
}


#[derive(Clone, PartialEq, Debug, Deserialize)]
struct User {
    name: String,
    email: String,
}


#[component]
fn App() -> Element {
    use_context_provider(|| Signal::<Option<User>>::new(None));
    let logged_in_user = use_context::<Signal<Option<User>>>();

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        Navbar {}

        div { class: "m-8",
            match &*logged_in_user.read() {
                Some(_user) => rsx! { ImageSetter {} },
                None => rsx! { ImageSetter {} },
            }
        }
        Login {}
    }
}

#[component]
fn Navbar() -> Element {
    rsx! {
        div { class: "navbar bg-base-300 shadow-xl p-8",
            div { class: "navbar-start" }
            div { class: "navbar-center",
                h1 { class: "text-4xl", "ARGA Heroes" }
            }
            div { class: "navbar-end",
                  UserDetails {}
            }
        }
    }
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

async fn login(form: LoginForm) -> Result<User, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .post(admin_api("login"))
        .json(&form)
        .send()
        .await?
        .json::<User>()
        .await
}


async fn get_main_image(scientific_name: &str) -> Result<Option<Photo>, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .get(admin_api("media/main"))
        .query(&[("scientific_name", scientific_name)])
        .send()
        .await?
        .json::<Option<Photo>>()
        .await
}

async fn set_main_image(scientific_name: &str, photo: &Photo) -> Result<Photo, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .post(admin_api("media/main"))
        .json(photo)
        .query(&[("scientific_name", scientific_name)])
        .send()
        .await?
        .json::<Photo>()
        .await
}


#[derive(Clone, PartialEq, Debug, Deserialize)]
struct TaxonName {
    scientific_name: String,
    canonical_name: String,
    authorship: Option<String>,
    rank: String,
}


async fn get_taxa(rank: &str, scientific_name: Option<&str>) -> Result<Vec<TaxonName>, reqwest::Error> {
    let rank = rank.to_lowercase();
    let next_rank = match rank.as_str() {
        "class" => "family",
        "family" => "genus",
        "genus" => "species",
        _ => "class",
    };

    let url = match scientific_name {
        Some(name) => format!("taxa/{next_rank}?parent={name}"),
        None => format!("taxa/{next_rank}"),
    };
    reqwest::get(admin_api(&url)).await?.json::<Vec<TaxonName>>().await
}


#[derive(Clone, PartialEq, Debug, Deserialize)]
struct PhotoPage {
    total: i32,
    page: i32,
    per_page: i32,
    photos: Vec<Photo>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct Photo {
    url: String,
    license: Option<String>,
    rights_holder: Option<String>,
    publisher: Option<String>,
    source: Option<String>,
}


#[derive(Clone, PartialEq, Debug, Deserialize)]
struct INaturalistMediaResponse {
    total_results: i32,
    page: i32,
    per_page: i32,
    results: Vec<INaturalistMediaItem>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
struct INaturalistMediaItem {
    photos: Vec<INaturalistMediaPhoto>,
    uri: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
struct INaturalistMediaPhoto {
    license_code: String,
    url: String,
    attribution: String,
}

impl From<INaturalistMediaResponse> for PhotoPage {
    fn from(value: INaturalistMediaResponse) -> Self {
        Self {
            total: value.total_results,
            page: value.page,
            per_page: value.per_page,
            photos: value
                .results
                .into_iter()
                .map(|item| Vec::<Photo>::from(item))
                .flatten()
                .collect(),
        }
    }
}

impl From<INaturalistMediaItem> for Vec<Photo> {
    fn from(value: INaturalistMediaItem) -> Self {
        value
            .photos
            .into_iter()
            .map(|photo| Photo {
                url: photo.url.replace("square", "medium"),
                license: Some(photo.license_code),
                rights_holder: Some(photo.attribution),
                publisher: None,
                source: Some(value.uri.clone()),
            })
            .collect()
    }
}


async fn get_inaturalist_media(scientific_name: &str) -> Result<PhotoPage, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .get("https://api.inaturalist.org/v1/observations")
        .query(&[
            ("photo_licensed", "true"),
            ("verifiable", "true"),
            ("quality_grade", "research"),
            ("order_by", "votes"),
            ("taxon_name", scientific_name),
            ("page", "1"),
            ("per_page", "20"),
        ])
        .send()
        .await?
        .json::<INaturalistMediaResponse>()
        .await
        .map(|resp| resp.into())
}


#[component]
fn Login() -> Element {
    let mut logged_in_user = use_context::<Signal<Option<User>>>();
    let mut login_form = use_signal(|| None);

    let login_resource = use_resource(move || async move {
        let response = match login_form() {
            Some(form) => Some(login(form).await),
            None => None,
        };

        if let Some(Ok(user)) = &response {
            *logged_in_user.write() = Some(user.clone());
        }

        response
    });

    rsx! {
        // there isn't a lazy version of resources so we need to unpack the resource twice.
        // the first is the 'in-flight' status of the resource, which returns Some after a response from
        // the closure is received.
        // the second is to determine if a login request has been attempted or not.
        div { match &*login_resource.read() {
            // login attempted and succeeded
            Some(Some(Ok(_))) => rsx! { button { r#type: "submit", "Logout" } },
            // login attempted and failed
            Some(Some(Err(err))) => rsx! { "Error: {err:#?}" },
            // no login attempted
            Some(None) => rsx! { "No login info" },
            // in-flight
            None => rsx! { "Attempting" },
        } }

        form {
            onsubmit: move |ev| login_form.set(Some(LoginForm {
                email: ev.values()["email"].as_value(),
                password: ev.values()["password"].as_value(),
            })),
            div { input { name: "email", placeholder: "Email", r#type: "text", class: "input" } }
            div { input { name: "password", placeholder: "Password", r#type: "password", class: "input" } }
            button { r#type: "submit", class: "btn btn-primary btn-wide btn-soft", "Login" }
        }
    }
}

#[component]
fn UserDetails() -> Element {
    let logged_in_user = use_context::<Signal<Option<User>>>();

    rsx! {
        match &*logged_in_user.read() {
            Some(user) => rsx! {
                p { "Welcome {user.name}" }
            },
            None => rsx! { p { "" } },
        }
    }
}


#[component]
fn ImageSetter() -> Element {
    let mut selected_species = use_signal(|| {
        Some(TaxonName {
            scientific_name: "Vombatus ursinus (Shaw, 1800)".to_string(),
            canonical_name: "Vombatus ursinus".to_string(),
            authorship: Some("(Shaw, 1800)".to_string()),
            rank: "species".to_string(),
        })
    });
    let mut selected_photo = use_signal(|| None);

    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-6 gap-4",
            div { TaxaList { rank: "classes", onspecies: move |name| selected_species.set(Some(name)) } }
            div { class: "md:col-span-2",
                if let Some(name) = selected_species() { MainImage { species: name, new_photo: selected_photo() } }
            }
            div { class: "md:col-span-3 md:col-start-4", ImageBrowser {
                species: selected_species(),
                onselected: move |photo| selected_photo.set(Some(photo)),
            } }
        }
    }
}

#[component]
fn ImageBrowser(species: Option<TaxonName>, onselected: EventHandler<Photo>) -> Element {
    let mut load_inat = use_signal(|| false);

    rsx! {
        div { class: "tabs tabs-box",
            input { type: "radio", name: "photos", class: "tab", aria_label: "Imported photos" }
            div { class: "tab-content bg-base-100 border-base-300 p-6",
                "ARGA loaded"
            }

            input { type: "radio", name: "photos", class: "tab", aria_label: "iNaturalist photos" }
            div { class: "tab-content bg-base-100 border-base-300 p-6",
                if let Some(name) = species {
                    match load_inat() {
                        true => rsx! { INaturalistMedia {
                            species: name.clone(),
                            onselected: move |photo: Photo| onselected.call(photo)
                            // move |photo| spawn(async move {set_main_image(&name.scientific_name, photo).await}),
                        } },
                        false => rsx! { button { class: "btn btn-block", onclick: move |_| load_inat.set(true), "Load photos" } },
                    }
                }
            }
        }
    }
}


#[component]
fn MainImage(species: ReadOnlySignal<TaxonName>, new_photo: ReadOnlySignal<Option<Photo>>) -> Element {
    let main_image = use_resource(move || async move { get_main_image(&*species.read().scientific_name).await });

    // let photo = photo.clone();
    // let name = name.clone();
    // spawn(async move {
    //     set_main_image(&name.scientific_name, &photo).await.unwrap();
    // });

    rsx! {
        h1 { class: "text-center", "Current hero photo" }
        match &*main_image.read() {
            Some(Ok(Some(photo))) => rsx! {
                div { class: "stack stack-top m-10",
                    if let Some(new_photo) = new_photo() {
                        img { class: "object-cover rotate-5 rounded-box shadow-md border border-base-content card", src: new_photo.url.clone() }
                    }
                    img { class: "object-cover rounded-box outline-4 shadow-xl border border-base-content card", src: photo.url.clone() }
                }
            },
            Some(Ok(None)) => rsx! { p { "No hero photo" } },
            Some(Err(err)) => rsx! { p { "Failed: {err:#?}" } },
            None => rsx! { p { "Loading..." } },
        }
    }
}


#[component]
fn INaturalistMedia(species: ReadOnlySignal<TaxonName>, onselected: EventHandler<Photo>) -> Element {
    let media = use_resource(move || async move { get_inaturalist_media(&*species.read().canonical_name).await });

    rsx! {
        match &*media.read() {
            Some(Ok(resp)) => rsx! {
                p { "Total: {resp.total}" }
                div { Photos { photos: resp.photos.clone(), onclick: move |photo| onselected.call(photo) } }
            },
            Some(Err(err)) => rsx! { p { "Failed: {err:#?}" } },
            None => rsx! { p { "Loading..." } },
        }
    }
}

#[component]
fn Photos(photos: Vec<Photo>, onclick: EventHandler<Photo>) -> Element {
    rsx! {
        div { class: "grid grid-cols-3 gap-4",
            for photo in photos {
                img { class: "cursor-pointer", src: "{photo.url}", onclick: move |_| onclick.call(photo.clone()) }
            }
        }
    }
}


#[component]
fn TaxaList(rank: ReadOnlySignal<String>, onspecies: EventHandler<TaxonName>) -> Element {
    let mut selected = use_signal::<Vec<TaxonName>>(|| vec![]);
    let mut filter = use_signal::<Option<String>>(|| None);

    let taxa = use_resource(move || async move {
        match selected.last() {
            Some(name) => get_taxa(&name.rank, Some(&name.scientific_name)).await,
            None => get_taxa("phylum", None).await,
        }
    });

    rsx! {
        match &*taxa.read() {
            Some(Ok(names)) => rsx! {
                div { class: "w-100% join join-vertical",
                    for name in selected() {
                        div { class: "join join-horizontal",
                            RightCaret {}
                            p { class: "text-xs pl-2", "{name.canonical_name}" }
                        }
                    }
                }

                div { TaxaFilter { onchange: move |val: String| if val.len() > 0 { filter.set(Some(val)) } else { filter.set(None) } } }
                div { class: "w-100%", p { class: "text-xs text-gray-500 text-right my-2", "Total: {names.len()}" } }
                div { class: "list overscroll-contain overflow-auto h-[calc(100vh-200px)]",
                    for name in names.iter().filter(|n| {
                        match filter() {
                            Some(needle) => n.canonical_name.to_lowercase().contains(&needle.to_lowercase()),
                            None => true,
                        }
                    }) {
                        TaxonListItem { item: name.clone(), onclick: move |name: TaxonName| {
                            filter.set(None);
                            if name.rank == "Species" {
                                onspecies.call(name);
                            } else {
                                selected.push(name);
                            }
                        }}
                    }
                }
            },
            Some(Err(err)) => rsx! { p { "Failed: {err:#?}" } },
            None => rsx! { p { "Loading..." } },
        }
    }
}


#[component]
fn TaxonListItem(item: TaxonName, onclick: EventHandler<TaxonName>) -> Element {
    rsx! {
        div { class: "list-row w-100% p-0",
            button {
                onclick: move |_| onclick.call(item.clone()),
                class: "p-0 list-col-grow btn btn-block btn-ghost btn-sm font-thin text-left",
                "{item.canonical_name}"
            }
            RightCaret {}
        }
    }
}

#[component]
fn RightCaret() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", width: "10px", height: "100%",
            g {
                stroke_linejoin: "round",
                stroke_linecap: "round",
                stroke_width: "2",
                fill: "none",
                stroke: "currentColor",
                path { d: "M6 3L20 12 6 21 6 3z" }
            }
        }
    }
}

#[component]
fn IconSearch() -> Element {
    rsx! {
        svg { class: "h-[1em] opacity-50", view_box: "0 0 24 24",
            g {
                stroke_linejoin: "round",
                stroke_linecap: "round",
                stroke_width: "2.5",
                fill: "none",
                stroke: "currentColor",
                circle { cx: "11", cy: "11", r: "8" },
                path { d: "m21 21-4.3-4.3" }
            }
        }
    }
}


#[component]
fn TaxaFilter(onchange: EventHandler<String>) -> Element {
    rsx! {
        label { class: "input",
            IconSearch {}
            input { type: "search", class: "grow", placeholder: "Filter", oninput: move |ev| onchange.call(ev.value()) }
        }
    }
}
