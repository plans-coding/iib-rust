use std::sync::Mutex;
use chrono::Local;
use once_cell::sync::OnceCell;
use serde_json::json;
use tera::Tera;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use opfs::{
    persistent::app_specific_dir,
    CreateWritableOptions, GetFileHandleOptions,
    DirectoryHandle, FileHandle, WritableFileStream,
};

mod sqlite_query;

macro_rules! define_resources {
    ($($name:ident => $path:expr),+ $(,)?) => {
        $( pub const $name: &str = include_str!($path); )+
        pub const ALL_QUERIES: &[(&str, &str)] = &[ $( define_resources!(@is_query $name) ),+ ];
    };
    (@is_query $name:ident) => { { (stringify!($name), $name) } };
}

define_resources! {

    CURRENT_VERSION => "../version",

    TEMPLATE_MENU => "../src/templates/_menu.tera",
    TEMPLATE_BREADCRUMBS => "../src/templates/_breadcrumbs.tera",
    TEMPLATE_EXPLORE => "../src/templates/explore.tera",
    TEMPLATE_OVERVIEW_YEAR => "../src/templates/overview_year.tera",
    TEMPLATE_OVERVIEW_COUNTRY => "../src/templates/overview_country.tera",
    TEMPLATE_OVERVIEW_PLAIN => "../src/templates/overview_plain.tera",
    TEMPLATE_TRIP => "../src/templates/trip.tera",
    TEMPLATE_IMAGES => "../src/templates/images.tera",
    TEMPLATE_MAP => "../src/templates/map.tera",
    TEMPLATE_STATISTICS_SUMMARY => "../src/templates/statistics_summary.tera",
    TEMPLATE_STATISTICS_VISITS => "../src/templates/statistics_visits.tera",
    TEMPLATE_STATISTICS_OVERNIGHTS => "../src/templates/statistics_overnights.tera",
    TEMPLATE_STATISTICS_THEMES => "../src/templates/statistics_themes.tera",
    TEMPLATE_DATASET => "../src/templates/dataset.tera",
    TEMPLATE_SOURCE => "../src/templates/source.tera",
    TEMPLATE_ABOUT => "../src/templates/about.tera",
    TEMPLATE_SEARCH => "../src/templates/search.tera",
    TEMPLATE_TOOLBOX_REPORT => "../src/templates/report.tera",
    TEMPLATE_TOOLBOX_REPORT_OUTPUT => "../src/templates/report_output.tera",
    TEMPLATE_TOOLBOX_INPUT => "../src/templates/toolbox.tera",

    QUERY_EXPLORE => "../src/queries/explore/explore.sql",
    QUERY_OVERVIEW_YEAR => "../src/queries/overview/overview_year.sql",
    QUERY_OVERVIEW_COUNTRY => "../src/queries/overview/overview_country.sql",
    QUERY_OVERVIEW_PLAIN => "../src/queries/overview/overview_plain.sql",
    QUERY_TRIP_PREVIOUS => "../src/queries/trip/trip_previous.sql",
    QUERY_TRIP_NEXT => "../src/queries/trip/trip_next.sql",
    QUERY_TRIP_SUMMARY => "../src/queries/trip/trip_summary.sql",
    QUERY_TRIP_EVENTS => "../src/queries/trip/trip_events.sql",
    QUERY_TRIP_UNIQUE_COUNTRIES => "../src/queries/trip/trip_unique_countries.sql",
    QUERY_TRIP_BORDER_CROSSINGS => "../src/queries/trip/trip_border_crossings.sql",
    QUERY_TRIP_MAP_PINS_ACCOMMODATION => "../src/queries/trip/trip_map_pins_accommodation.sql",
    QUERY_TRIP_MAP_PINS_OVERALL => "../src/queries/trip/trip_map_pins_overall.sql",
    QUERY_TRIP_IMMICH_DESC_SEARCH => "../src/queries/trip/trip_immich_desc_search.sql",
    QUERY_TRIP_IMMICH_ALBUM_NAME => "../src/queries/trip/trip_immich_album_name.sql",
    QUERY_TRIP_EXTENSION_MOVIE => "../src/queries/_extensions/trip_movie.sql",
    QUERY_STATISTICS_VISITS => "../src/queries/statistics/statistics_visits.sql",
    QUERY_STATISTICS_OVERNIGHTS => "../src/queries/statistics/statistics_overnights.sql",
    QUERY_STATISTICS_PER_DOMAIN_YEAR => "../src/queries/statistics/statistics_per_domain_year.sql",
    QUERY_STATISTICS_THEME_COUNT => "../src/queries/statistics/statistics_theme_count.sql",
    QUERY_STATISTICS_TRIP_COUNT => "../src/queries/statistics/statistics_trip_count.sql",
    QUERY_COMMON_PARTICIPANT_GROUPS => "../src/queries/_common/common_participant_groups.sql",
    QUERY_COMMON_TRIP_DOMAINS => "../src/queries/_common/common_trip_domains.sql",
    QUERY_COMMON_TRIP_LABELS => "../src/queries/_common/common_trip_labels.sql",
    QUERY_IMAGES_DATE_LIST => "../src/queries/images/images_date_list.sql",
    QUERY_IMAGES_PHOTO_TIME => "../src/queries/images/images_photo_time.sql",
    QUERY_MAP_CONTOUR => "../src/queries/map/map_contour.sql",
    QUERY_MAP_COUNTRY => "../src/queries/map/map_country.sql",
    QUERY_MAP_COUNTRY_LIST => "../src/queries/map/map_country_list.sql",
    QUERY_MAP_THEME => "../src/queries/map/map_theme.sql",
    QUERY_SEARCH_EVENT => "../src/queries/search/search_event.sql",
    QUERY_SEARCH_TRIP => "../src/queries/search/search_trip.sql",
    QUERY_REPORT_ALL_OVERVIEW => "../src/queries/report/report_all_overview.sql",
    QUERY_REPORT_ALL_EVENTS => "../src/queries/report/report_all_events.sql",

    CHART_JS => "../bundle/chartjs/chart.js",
    //MAPLIBRE_JS => "../bundle/maplibre-gl/maplibre-gl.js",
    //MAPLIBRE_CSS => "../bundle/maplibre-gl/maplibre-gl.css",
    //BEWGUNG_CSS => "../bewegung.css",

}

static TERA: OnceCell<Tera> = OnceCell::new();

fn get_tera() -> &'static Tera {
    TERA.get_or_init(|| {
        let mut tera = Tera::default();
        
        let bc = |page: &str, body: &str| format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), body);

        let templates = [
            ("menu", TEMPLATE_MENU.to_string()),
            ("breadcrumbs", TEMPLATE_BREADCRUMBS.to_string()),
            ("explore", bc("explore", TEMPLATE_EXPLORE)),
            ("overview_year", bc("overview:year", TEMPLATE_OVERVIEW_YEAR)),
            ("overview_country", bc("overview:country", TEMPLATE_OVERVIEW_COUNTRY)),
            ("overview_plain", bc("overview:plain", TEMPLATE_OVERVIEW_PLAIN)),
            ("statistics_summary", bc("statistics:summary", TEMPLATE_STATISTICS_SUMMARY)),
            ("statistics_visits", bc("statistics:visits", TEMPLATE_STATISTICS_VISITS)),
            ("statistics_overnights", bc("statistics:overnights", TEMPLATE_STATISTICS_OVERNIGHTS)),
            ("statistics_themes", bc("statistics:themes", TEMPLATE_STATISTICS_THEMES)),
            ("source", bc("source", TEMPLATE_SOURCE)),
            ("about", bc("about", TEMPLATE_ABOUT)),
            ("report", bc("report", TEMPLATE_TOOLBOX_REPORT)),
            ("trip", TEMPLATE_TRIP.to_string()),
            ("images", TEMPLATE_IMAGES.to_string()),
            ("map", TEMPLATE_MAP.to_string()),
            ("dataset", TEMPLATE_DATASET.to_string()),
            ("search", TEMPLATE_SEARCH.to_string()),
            ("report_output", TEMPLATE_TOOLBOX_REPORT_OUTPUT.to_string()),
            ("toolbox", TEMPLATE_TOOLBOX_INPUT.to_string()),
        ];

        for (name, content) in templates {
            if let Err(e) = tera.add_raw_template(name, &content) {
                web_sys::console::error_1(&format!("Tera Error in {name}: {e}").into());
            }
        }
        tera
    })
}

static DB_BYTES: OnceCell<Vec<u8>> = OnceCell::new();
static RENDER_STRUCTURE: OnceCell<Mutex<tera::Context>> = OnceCell::new();

// -----------------------------------------------------------------------
// MAKE JAVASCRIPT FUNCTIONS AVAILABLE FOR RUST
// -----------------------------------------------------------------------
#[wasm_bindgen]
extern "C" {
    // Charts
    fn initializeChart();
    fn initializeChartOvernights();
    // Maps
    fn load_trip_map();
    fn load_contour_map();
    fn load_country_map();
    fn load_theme_map();
    //Other
    fn load_code_editor();
    fn initiate_spreadsheet();
    fn custom_queries();
    //fn inject_css(css: &str);
    fn initialize_theme_color();
    fn check_immich_authorization();
    fn init_create_trip();
    fn load_filter_OPFS();

    #[wasm_bindgen(catch)]
    async fn get_filter_value_OPFS() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn check_available_update() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn read_opfs_file(path: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn read_opfs_text(path: &str) -> Result<JsValue, JsValue>;
}

// -----------------------------------------------------------------------
// REAL WASM START
// -----------------------------------------------------------------------
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let (db_bytes, render_structure) = session_load().await;

    DB_BYTES.set(db_bytes).expect("DB already initialized");
    RENDER_STRUCTURE
        .set(Mutex::new(render_structure))
        .expect("Render structure already initialized");

    page_load_internal().await;
    Ok(())
}

// -----------------------------------------------------------------------
// MAKE RUST FUNCTIONS AVAILABLE FOR JAVASCRIPT
// -----------------------------------------------------------------------
#[wasm_bindgen]
pub async fn page_load() {
    page_load_internal().await;
}

#[wasm_bindgen(getter)]
pub fn chart_js() -> String {
    // .to_string() or .into() converts &str to String
    CHART_JS.to_string()
}

#[wasm_bindgen]
pub fn get_predefined_query(name: &str) -> Option<String> {
    ALL_QUERIES
        .iter()
        .find(|&&(k, _)| k == name)
        .map(|&(_, v)| v.to_string())
}

#[wasm_bindgen]
pub fn user_run_sql(sql: String) -> js_sys::Promise {
    future_to_promise(async move {
        sqlite_query::user_run_sql_internal(sql).await;
        Ok(JsValue::UNDEFINED)
    })
}

// -----------------------------------------------------------------------
// INITIATE SESSION
// -----------------------------------------------------------------------

async fn session_load() -> (Vec<u8>, tera::Context) {
    let db_bytes = get_sqlite_binary().await;
    if !db_bytes.is_empty() { 
        sqlite_query::get_or_init_db(&db_bytes); 
    }

    // 1. Get current path from URL
    let path = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok())
        .and_then(|p| p.get("path"))
        .unwrap_or_else(|| "explore".to_string());

    // 2. Query Database
    let mut results = sqlite_query::get_query_data_universal(&db_bytes, vec![
        ("file".into(), "SELECT Value FROM bewx_Settings WHERE Attribute = 'LanguageFile' LIMIT 1;".into()),
        ("settings".into(), "SELECT * FROM bewx_Settings;".into()),
        ("common_trip_domains".into(), QUERY_COMMON_TRIP_DOMAINS.into()),
        ("common_participant_groups".into(), QUERY_COMMON_PARTICIPANT_GROUPS.into()),
        ("common_trip_labels".into(), QUERY_COMMON_TRIP_LABELS.into()),
    ], false).await;

    // 3. Fetch Translation File
    let translation = match results.pointer("/file/0/Value").and_then(|v| v.as_str()) {
        Some(f) => {
            let url = format!("static/languages/{}", f);
            
            // Execute the fetch logic
            async {
                let window = web_sys::window()?;
                let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(&url)).await.ok()?;
                let response: web_sys::Response = resp_value.dyn_into().ok()?;
                
                if !response.ok() {
                    return None;
                }

                let text = wasm_bindgen_futures::JsFuture::from(response.text().ok()?).await.ok()?;
                let text_str = text.as_string()?;
                serde_json::from_str(&text_str).ok()
            }
            .await
            .unwrap_or_else(|| serde_json::json!({}))
        }
        None => serde_json::json!({}),
    };

    // 4. Build Settings Map (Grouping by AttributeGroup)
    let mut settings_map = serde_json::Map::new();
    if let Some(arr) = results["settings"].as_array() {
        for s in arr {
            let group = s["AttributeGroup"].as_str().unwrap_or("General");
            let attr = s["Attribute"].as_str().unwrap_or("Unknown");
            
            settings_map.entry(group)
                .or_insert(serde_json::json!({}))
                .as_object_mut()
                .unwrap()
                .insert(attr.to_string(), s["Value"].clone());
        }
    }

    // Clean up internal query metadata before moving to common
    if let Some(obj) = results.as_object_mut() {
        obj.remove("settings");
        obj.remove("file");
    }

    // 5. Build Tera Context directly
    let mut context = tera::Context::new();
    context.insert("path", &path);
    context.insert("translation", &translation);
    context.insert("settings", &settings_map);
    context.insert("common", &results);

    // 6. Direct Rendering Logic
    let render_result = (|| -> Result<(), String> {
        let rendered = get_tera()
            .render("menu", &context)
            .map_err(|e| format!("Render error: {e}"))?;

        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("menu"))
            .ok_or_else(|| "Element #menu not found".to_string())?
            .set_inner_html(&rendered);
        
        Ok(())
    })();

    if let Err(e) = render_result {
        web_sys::console::log_1(&e.into());
        // Optional: Update an error element in the DOM
    }

    initialize_theme_color();
    
    (db_bytes, context)
}   

// -----------------------------------------------------------------------
// HOT RELOAD
// -----------------------------------------------------------------------
async fn page_load_internal() {
    //web_sys::console::log_1(&">>----------------------".into());

    let db_bytes = DB_BYTES.get().expect("DB not initialized");
    let render_structure_mutex = RENDER_STRUCTURE.get().expect("Render structure missing");

    // Lock the Mutex to get a mutable reference
    let mut render_structure = render_structure_mutex.lock().expect("ERROR");
    let mut all_state = render_structure
        .get("all")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if !all_state.is_object() {
        all_state = json!({});
    }

    let path = web_sys::window()
        .expect("No window available")
        .location()
        .search()
        .ok()
        .as_deref()
        .and_then(|s| web_sys::UrlSearchParams::new_with_str(s).ok())
        .and_then(|params| params.get("path"));

    let page = path.as_deref().unwrap_or("explore");

    //web_sys::console::log_1(&format!("Loading page: {}",page).into());

    all_state["query_params"] = json!({ "path": path.clone() });

    // READ APPLIED FILTERS  -----------------------------------------------------------------------

    let filter_values = get_filter_value_OPFS().await.unwrap();
    all_state["filters"] = serde_wasm_bindgen::from_value(filter_values).unwrap();

    // Prepare filters
    fn format_sql_in(filter: &serde_json::Value, fallback: &str) -> String {
        filter.as_array()
            .filter(|a| !a.is_empty())
            .map(|arr| {
                let joined = arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| format!("'{}'", s.replace('\'', "''"))) // Added basic escaping
                    .collect::<Vec<_>>()
                    .join(",");
                format!("({})", joined)
            })
            .unwrap_or_else(|| fallback.to_string())
    }

    let filters = &all_state["filters"];

    let participant_group = format_sql_in(&filters["participantGroup"], "(ParticipantGroup)");
    let trip_domain       = format_sql_in(&filters["tripDomain"], "(TripDomain)");

    let trip_label = filters["tripLabel"].as_array()
        .filter(|a| !a.is_empty())
        .map(|arr| {
            let conds = arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| format!("(',' || REPLACE(TripLabels,' ','') || ',') LIKE '%,{},%'", s.replace('\'', "''")))
                .collect::<Vec<_>>()
                .join(" AND ");
            format!("({})", conds)
        })
        .unwrap_or_else(|| "1=1".to_string());

    let tr = |ptr: &str, fallback: &str| -> String {
        render_structure
            .get("translation")
            .and_then(|v| v.pointer(ptr))
            .and_then(|v| v.as_str())
            .unwrap_or(fallback)
            .to_string()
    };
    let settings_text = |ptr: &str, fallback: &str| -> String {
        render_structure
            .get("settings")
            .and_then(|v| v.pointer(ptr))
            .and_then(|v| v.as_str())
            .unwrap_or(fallback)
            .to_string()
    };

    use serde_json;
    use std::collections::HashMap;
    
    let cover_photos_json = read_opfs_text("cover_photos.json").await.ok();

    let cover_photos_map: HashMap<String, String> = cover_photos_json
    .and_then(|val| {
        if val.is_null() || val.is_undefined() {
            None
        } else {
            val.as_string()
        }
    })
    .and_then(|json_str| {
        serde_json::from_str(&json_str).map_err(|e| {
            web_sys::console::log_1(&format!("JSON Parse Error: {:?}", e).into());
        }).ok()
    })
    .unwrap_or_default();


    // APLLY FILTER FUNCTION

    trait QueryFilters {
        fn apply_filters(&self, group: &str, domain: &str, label: &str) -> String;
    }

    impl QueryFilters for str {
        fn apply_filters(&self, group: &str, domain: &str, label: &str) -> String {
            self.replace("/*", "")
                .replace("*/", "")
                .replace("(ParticipantGroup)", group)
                .replace("(TripDomain)", domain)
                .replace("1=1", label)
        }
    }

    use serde::Serialize;

    #[derive(Serialize, Debug, Clone)]
    pub struct PageData {
        pub title: String,
        pub template: String,
        pub queries: Vec<(String, String)>,
    }

    // -----------------------------------------------------------------------
    // Fourth: Page specific data
    // -----------------------------------------------------------------------
    let mut page_data: Option<PageData> = None;
    let mut execute_after: Vec<String> = Vec::new();
    match page {
        "explore" => {
            page_data = Some(PageData {
                title: tr("/explore/title", "Explore"),
                template: "explore".into(),
                queries: vec![(
                    "explore".into(),
                    QUERY_EXPLORE.apply_filters(&participant_group, &trip_domain, &trip_label),
                )],
            });
            all_state["cover_photos_list"] =
                serde_json::to_value(&cover_photos_map).expect("Failed to convert map to Value");
        }
        "overview:year" => {
            page_data = Some(PageData {
                title: tr("/overview/year", "Overview: Year"),
                template: "overview_year".into(),
                queries: vec![(
                    "overview_year".into(),
                    QUERY_OVERVIEW_YEAR.apply_filters(
                        &participant_group,
                        &trip_domain,
                        &trip_label,
                    ),
                )],
            });
        }
        "overview:country" => {
            page_data = Some(PageData {
                title: tr("/overview/country", "Overview: Country"),
                template: "overview_country".into(),
                queries: vec![(
                    "overview_country".into(),
                    QUERY_OVERVIEW_COUNTRY.apply_filters(
                        &participant_group,
                        &trip_domain,
                        &trip_label,
                    ),
                )],
            });
        }

        "overview:plain" => {
            page_data = Some(PageData {
                title: tr("/overview/plain", "Overview: Plain"),
                template: "overview_plain".into(),
                queries: vec![(
                    "overview_plain".into(),
                    QUERY_OVERVIEW_PLAIN.apply_filters(
                        &participant_group,
                        &trip_domain,
                        &trip_label,
                    ),
                )],
            });
        }
        "map" => {
            page_data = Some(PageData {
                title: tr("/map/title", "Map"),
                template: "map".into(),
                queries: vec![
                    (
                        "map_country_list".into(),
                        QUERY_MAP_COUNTRY_LIST.apply_filters(
                            &participant_group,
                            &trip_domain,
                            &trip_label,
                        ),
                    ),
                    (
                        "map_data".into(),
                        QUERY_MAP_CONTOUR.apply_filters(
                            &participant_group,
                            &trip_domain,
                            &trip_label,
                        ),
                    ),
                    (
                        "common_trip_domains".into(),
                        QUERY_COMMON_TRIP_DOMAINS.to_string(),
                    ),
                ],
            });
            execute_after = vec!["load_contour_map".to_string()];
        }
        "statistics:summary" => {
            page_data = Some(PageData {
                    title: tr("/statistics/summary", "Statistics: Summary"),
                    template: "statistics_summary".into(),
                    queries: vec![
                        ("statistics_visits".into(), QUERY_STATISTICS_VISITS.replace("SELECT\n    Country,\n    OL,\n    SS,\n    VSS,\n    PS,\n    OLMQ,\n    SSMQ,\n    VSSMQ,\n    PSMQ\nFROM Aggregated\nORDER BY OL DESC;", "SELECT COUNT(DISTINCT Country) AS TripCount FROM Aggregated;").replace("/*", "").replace("*/", "").apply_filters(&participant_group, &trip_domain, &trip_label)),
                        ("statistics_trip_count".into(), QUERY_STATISTICS_TRIP_COUNT.apply_filters(&participant_group, &trip_domain, &trip_label)),
                        ("statistics_per_domain_year".into(), QUERY_STATISTICS_PER_DOMAIN_YEAR.apply_filters(&participant_group, &trip_domain, &trip_label)),
                        ("common_trip_domains".into(), QUERY_COMMON_TRIP_DOMAINS.to_string()),
                    ],
                });
            execute_after = vec!["initializeChart".to_string()];
        }
        "statistics:visits" => {
            page_data = Some(PageData {
                title: tr("/statistics/visits", "Statistics: Visits"),
                template: "statistics_visits".into(),
                queries: vec![(
                    "statistics_visits".into(),
                    QUERY_STATISTICS_VISITS
                        .replace("/*", "")
                        .replace("*/", "")
                        .apply_filters(&participant_group, &trip_domain, &trip_label),
                )],
            });
        }
        "statistics:overnights" => {
            page_data = Some(PageData {
                title: tr("/statistics/overnights", "Statistics: Overnights"),
                template: "statistics_overnights".into(),
                queries: vec![(
                    "statistics_overnights".into(),
                    QUERY_STATISTICS_OVERNIGHTS
                        .replace("/*", "")
                        .replace("*/", "")
                        .apply_filters(&participant_group, &trip_domain, &trip_label),
                )],
            });
            execute_after = vec!["initializeChartOvernights".to_string()];
        }
        "statistics:themes" => {
            page_data = Some(PageData {
                title: settings_text("/Plugin/Theme/translation", "Themes"),
                template: "statistics_themes".into(),
                queries: vec![(
                    "statistics_theme_count".into(),
                    QUERY_STATISTICS_THEME_COUNT.apply_filters(
                        &participant_group,
                        &trip_domain,
                        &trip_label,
                    ),
                )],
            });
        }
        "dataset" => {
            page_data = Some(PageData {
                    title: tr("/dataset/title", "Dataset"),
                    template: "dataset".into(),
                    queries: vec![
                        ("table_list".into(), "SELECT name FROM sqlite_master WHERE type IN ('table', 'view') ORDER BY name;".to_string()),
                        ("stored_custom_queries".into(), "SELECT ROWID, Name FROM com_CodeCollection WHERE Target = 'BewDataset';".to_string()),
                    ],
                });
            all_state["query_templates"] =
                serde_json::json!(ALL_QUERIES.iter().map(|(name, _)| name).collect::<Vec<_>>());
            execute_after = vec![
                "load_code_editor".to_string(),
                "initiate_spreadsheet".to_string(),
            ];
        }
        "source" => {
            page_data = Some(PageData {
                title: tr("/source/title", "Source"),
                template: "source".into(),
                queries: vec![(
                    "cover_photo_original_paths".into(),
                    "SELECT OuterId, CoverPhoto FROM bewa_Overview WHERE CoverPhoto IS NOT NULL;"
                        .to_string(),
                )],
            });
            all_state["db_loaded"] = serde_json::json!(if !db_bytes.is_empty() {
                "stored"
            } else {
                "missing"
            });
            execute_after = vec!["check_immich_authorization".to_string()];
        }
        "about" => {
            page_data = Some(PageData {
                title: tr("/about/title", "About"),
                template: "about".into(),
                queries: vec![],
            });
            let update_info = check_available_update()
                .await
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            all_state["update_info"] = serde_json::json!(update_info);
            all_state["current_version"] = serde_json::json!(CURRENT_VERSION);
        }
        "report" => {
            page_data = Some(PageData {
                title: tr("/toolbox/report", "Report"),
                template: "report".into(),
                queries: vec![],
            });
            all_state["date_now"] = serde_json::json!(Local::now().format("%Y-%m-%d").to_string());
            all_state["year_now"] = serde_json::json!(Local::now().format("%Y").to_string());
        }
        "input" => {
            page_data = Some(PageData {
                title: tr("/toolbox/input", "Input"),
                template: "toolbox".into(),
                queries: vec![],
            });
            execute_after = vec!["init_create_trip".to_string()];
        }
        _ => {
            if let Some(rest) = page.strip_prefix("trip:") {
                let mut parts = rest.splitn(2, ':');
                let outer_id = parts.next().filter(|s| !s.is_empty());
                let inner_id = parts.next().filter(|s| !s.is_empty());

                match (outer_id, inner_id) {
                    (None, Some(inner_id)) => {
                        // TODO: REMOVE AND REDIRECT
                        page_data = Some(PageData {
                            title: inner_id.to_string(),
                            template: "trip".into(),
                            queries: vec![
                                (
                                    "trip_summary".into(),
                                    QUERY_TRIP_SUMMARY
                                        .replace("= InnerId", &format!("= '{}'", inner_id)),
                                ),
                                (
                                    "trip_events".into(),
                                    QUERY_TRIP_EVENTS
                                        .replace("= e.InnerId", &format!("= '{}'", inner_id)),
                                ),
                                (
                                    "common_trip_domains".into(),
                                    QUERY_COMMON_TRIP_DOMAINS.to_string(),
                                ),
                                (
                                    "trip_unique_countries".into(),
                                    QUERY_TRIP_UNIQUE_COUNTRIES
                                        .replace("= InnerId", &format!("= '{}'", inner_id)),
                                ),
                                (
                                    "trip_border_crossings".into(),
                                    QUERY_TRIP_BORDER_CROSSINGS
                                        .replace("= a.InnerId", &format!("= '{}'", inner_id)),
                                ),
                                (
                                    "trip_map_pins_overall".into(),
                                    QUERY_TRIP_MAP_PINS_OVERALL
                                        .replace("= InnerId", &format!("= '{}'", inner_id)),
                                ),
                                (
                                    "trip_map_pins_accommodation".into(),
                                    QUERY_TRIP_MAP_PINS_ACCOMMODATION
                                        .replace("= o.InnerId", &format!("= '{}'", inner_id)),
                                ),
                                (
                                    "trip_previous".into(),
                                    QUERY_TRIP_PREVIOUS
                                        .replace("= InnerId", &format!("= '{}'", inner_id))
                                        .apply_filters(
                                            &participant_group,
                                            &trip_domain,
                                            &trip_label,
                                        ),
                                ),
                                (
                                    "trip_next".into(),
                                    QUERY_TRIP_NEXT
                                        .replace("= InnerId", &format!("= '{}'", inner_id))
                                        .apply_filters(
                                            &participant_group,
                                            &trip_domain,
                                            &trip_label,
                                        ),
                                ),
                                (
                                    "trip_immich_desc_search".into(),
                                    QUERY_TRIP_IMMICH_DESC_SEARCH
                                        .replace("= InnerId", &format!("= '{}'", inner_id)),
                                ),
                                (
                                    "trip_immich_album_name".into(),
                                    QUERY_TRIP_IMMICH_ALBUM_NAME
                                        .replace("= InnerId", &format!("= '{}'", inner_id)),
                                ),
                            ],
                        });
                        all_state["cover_photos_list"] = serde_json::to_value(&cover_photos_map)
                            .expect("Failed to convert map to Value");
                        execute_after = vec!["load_trip_map".to_string()];
                    }
                    (Some(outer_id), _) => {
                        page_data = Some(PageData {
                            title: outer_id.to_string(),
                            template: "trip".into(),
                            queries: vec![
                                (
                                    "trip_summary".into(),
                                    QUERY_TRIP_SUMMARY
                                        .replace("= OuterId", &format!("= '{}'", outer_id)),
                                ),
                                (
                                    "trip_events".into(),
                                    QUERY_TRIP_EVENTS
                                        .replace("= o.OuterId", &format!("= '{}'", outer_id)),
                                ),
                                (
                                    "common_trip_domains".into(),
                                    QUERY_COMMON_TRIP_DOMAINS.to_string(),
                                ),
                                (
                                    "trip_unique_countries".into(),
                                    QUERY_TRIP_UNIQUE_COUNTRIES
                                        .replace("= OuterId", &format!("= '{}'", outer_id)),
                                ),
                                (
                                    "trip_border_crossings".into(),
                                    QUERY_TRIP_BORDER_CROSSINGS
                                        .replace("= b.OuterId", &format!("= '{}'", outer_id)),
                                ),
                                (
                                    "trip_map_pins_overall".into(),
                                    QUERY_TRIP_MAP_PINS_OVERALL
                                        .replace("= OuterId", &format!("= '{}'", outer_id)),
                                ),
                                (
                                    "trip_map_pins_accommodation".into(),
                                    QUERY_TRIP_MAP_PINS_ACCOMMODATION
                                        .replace("= o.OuterId", &format!("= '{}'", outer_id)),
                                ),
                                (
                                    "trip_previous".into(),
                                    QUERY_TRIP_PREVIOUS
                                        .replace("= OuterId", &format!("= '{}'", outer_id))
                                        .apply_filters(
                                            &participant_group,
                                            &trip_domain,
                                            &trip_label,
                                        ),
                                ),
                                (
                                    "trip_next".into(),
                                    QUERY_TRIP_NEXT
                                        .replace("= OuterId", &format!("= '{}'", outer_id))
                                        .apply_filters(
                                            &participant_group,
                                            &trip_domain,
                                            &trip_label,
                                        ),
                                ),
                                (
                                    "trip_immich_desc_search".into(),
                                    QUERY_TRIP_IMMICH_DESC_SEARCH
                                        .replace("= OuterId", &format!("= '{}'", outer_id)),
                                ),
                                (
                                    "trip_immich_album_name".into(),
                                    QUERY_TRIP_IMMICH_ALBUM_NAME
                                        .replace("= OuterId", &format!("= '{}'", outer_id)),
                                ),
                                (
                                    "trip_extension_movie".into(),
                                    QUERY_TRIP_EXTENSION_MOVIE.replace("_OUTER_ID_", outer_id),
                                ),
                            ],
                        });
                        all_state["cover_photos_list"] = serde_json::to_value(&cover_photos_map)
                            .expect("Failed to convert map to Value");
                        execute_after = vec!["load_trip_map".to_string()];
                    }
                    _ => {}
                }
            }

            if let Some(suffix) = page.strip_prefix("images:") {
                let mut parts = suffix.splitn(2, ':');
                if let (Some(trip_id), Some(trip_date)) = (parts.next(), parts.next()) {
                    page_data = Some(PageData {
                        title: suffix.to_string(),
                        template: "images".into(),
                        queries: vec![
                            (
                                "images_date_list".into(),
                                QUERY_IMAGES_DATE_LIST.replace("/*_OUTER_ID_*/", trip_id),
                            ),
                            (
                                "common_trip_domains".into(),
                                QUERY_COMMON_TRIP_DOMAINS.to_string(),
                            ),
                            (
                                "images_photo_time".into(),
                                QUERY_IMAGES_PHOTO_TIME.replace("/*_OUTER_ID_*/", trip_id),
                            ),
                        ],
                    });
                    all_state["trip_date"] = serde_json::json!(trip_date);
                    all_state["trip_id"] = serde_json::json!(trip_id);
                }
            }

            if let Some(suffix) = page.strip_prefix("map:") {
                let mut queries = vec![
                    (
                        "map_country_list".into(),
                        QUERY_MAP_COUNTRY_LIST.apply_filters(
                            &participant_group,
                            &trip_domain,
                            &trip_label,
                        ),
                    ),
                    (
                        "common_trip_domains".into(),
                        QUERY_COMMON_TRIP_DOMAINS.to_string(),
                    ),
                ];

                if let Some(country) = suffix.strip_prefix("country:") {
                    queries.push((
                        "map_data".into(),
                        QUERY_MAP_COUNTRY
                            .replace("_COUNTRY_", country)
                            .apply_filters(&participant_group, &trip_domain, &trip_label),
                    ));
                    page_data = Some(PageData {
                        title: tr("/map/title", "Map"),
                        template: "map".into(),
                        queries,
                    });
                    execute_after = vec!["load_country_map".to_string()];
                } else if let Some(theme) = suffix.strip_prefix("theme:") {
                    queries.push((
                        "map_data".into(),
                        QUERY_MAP_THEME.replace("_THEME_", theme).apply_filters(
                            &participant_group,
                            &trip_domain,
                            &trip_label,
                        ),
                    ));
                    page_data = Some(PageData {
                        title: tr("/map/title", "Map"),
                        template: "map".into(),
                        queries,
                    });
                    execute_after = vec!["load_theme_map".to_string()];
                }
            }

            if let Some(suffix) = page.strip_prefix("search:") {
                page_data = Some(PageData {
                    title: suffix.to_string(),
                    template: "search".into(),
                    queries: vec![
                        (
                            "search_trip".into(),
                            QUERY_SEARCH_TRIP
                                .replace("/*_STRING_*/", suffix)
                                .apply_filters(&participant_group, &trip_domain, &trip_label),
                        ),
                        (
                            "search_event".into(),
                            QUERY_SEARCH_EVENT
                                .replace("/*_STRING_*/", suffix)
                                .apply_filters(&participant_group, &trip_domain, &trip_label),
                        ),
                    ],
                });
            }

            if let Some(suffix) = page.strip_prefix("report:output:") {
                let mut parts = suffix.splitn(2, ':');
                if let (Some(title_str), Some(backside_str)) = (parts.next(), parts.next()) {
                    page_data = Some(PageData {
                        title: suffix.to_string(),
                        template: "report_output".into(),
                        queries: vec![
                            (
                                "all_overview".into(),
                                QUERY_REPORT_ALL_OVERVIEW.apply_filters(
                                    &participant_group,
                                    &trip_domain,
                                    &trip_label,
                                ),
                            ),
                            (
                                "all_events".into(),
                                QUERY_REPORT_ALL_EVENTS.apply_filters(
                                    &participant_group,
                                    &trip_domain,
                                    &trip_label,
                                ),
                            ),
                        ],
                    });
                    all_state["title_string"] = serde_json::json!(title_str);
                    all_state["backside_string"] = serde_json::json!(backside_str);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Fifth: Render content
    // -----------------------------------------------------------------------

    let Some(page_data) = page_data else {
        return;
    };

    // SET TITLE  -----------------------------------------------------------------------
    web_sys::window()
        .expect("ERROR")
        .document()
        .expect("ERROR")
        .set_title(&format!("{} - Immer in Bewegung", page_data.title));

    // RUN SQLITE QUERIES  -----------------------------------------------------------------------

    let query_response: serde_json::Value =
        sqlite_query::get_query_data_universal(&db_bytes, page_data.queries.clone(), false).await;

    // RENDER TO 'APP'  -----------------------------------------------------------------------

    render_structure.insert("all", &all_state);

    let mut context = render_structure.clone();
    context.insert("all", &all_state);
    context.insert("query_params", &all_state["query_params"]);
    //context.insert("title", &page_data.title);

    if let Some(obj) = query_response.as_object() {
        for (key, value) in obj {
            context.insert(key, value);
        }
    }

    for key in [
        "trip_date",
        "trip_id",
        "title_string",
        "backside_string",
        "update_info",
    ] {
        if let Some(value) = all_state.get(key) {
            context.insert(key, value);
        }
    }

    let render_result = (|| -> Result<(), String> {
        let rendered = get_tera()
            .render(&page_data.template, &context)
            .map_err(|e| format!("Render error: {e}"))?;

        /*if include_wrapper {
            let external_map_provider = json_object
            .get("settings")
            .and_then(|s| s.get("Other"))
            .and_then(|b| b.get("ExternalMapProvider"))
            .and_then(|v| v.as_str())
            .ok_or("Missing ExternalMapProvider")?;

            let re = Regex::new(r"\{([^}]*)\}\(([^)]*)\)\[([^\]]*)\]")
            .map_err(|e| format!("Regex error: {e:?}"))?;

            rendered = re
            .replace_all(&rendered, |caps: &regex::Captures| {
                format!(
                    "<a target=\"_blank\" class=\"theme-link\" href=\"{}{}\">{}</a>",
                    external_map_provider, &caps[2], &caps[1]
                )
                })
                .into_owned();
        }*/

        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| "Element #app not found".to_string())?
            .set_inner_html(&rendered);

        Ok(())
    })();

    if let Err(e) = render_result {
        web_sys::console::log_1(&e.into());
    }



    // RUN FUNCTIONS AFTER PAGE LOAD  ------------------------------------

    load_filter_OPFS();

    for action in execute_after {
        match action.as_str() {
            "load_trip_map" => load_trip_map(),
            "load_contour_map" => load_contour_map(),
            "load_country_map" => load_country_map(),
            "load_theme_map" => load_theme_map(),
            "load_code_editor" => load_code_editor(),
            "initiate_spreadsheet" => initiate_spreadsheet(),
            "initializeChart" => initializeChart(),
            "initializeChartOvernights" => initializeChartOvernights(),
            "check_immich_authorization" => check_immich_authorization(),
            "init_create_trip" => init_create_trip(),
            "about" => {}
            _ => {}
        }
    }

}

pub async fn get_sqlite_binary() -> Vec<u8> {
    // 1) Check if file exists in OPFS
    if let Ok(value) = read_opfs_file("chronik.sqlite").await {
        if !value.is_null() && !value.is_undefined() {
            web_sys::console::log_1(&"Found chronik.sqlite in OPFS".into());
            return js_sys::Uint8Array::new(&value).to_vec();
        }
    }

    web_sys::console::log_1(&"No chronik.sqlite → checking server fallback...".into());

    // 2) Load server_db_path.txt
    let server_path = {
        let window = web_sys::window().expect("no global `window` exists");
        let fetch_task = async {
            let response_value = JsFuture::from(window.fetch_with_str("server_db_path.txt")).await.ok()?;
            let response: web_sys::Response = response_value.dyn_into().ok()?;
            let text = JsFuture::from(response.text().ok()?).await.ok()?;
            text.as_string()
        };

        match fetch_task.await {
            Some(path) if !path.trim().is_empty() => path.trim().to_string(),
            _ => {
                web_sys::console::log_1(&"server_db_path.txt missing, empty, or fetch failed".into());
                return Vec::new(); // Or handle error appropriately
            }
        }
    };

    web_sys::console::log_1(&format!("Server path: {}", server_path).into());

    // Download actual DB file
    let db_bytes = match {
        let url = &server_path;
        async {
            let window = web_sys::window()?;
            let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url)).await.ok()?;
            let response: web_sys::Response = resp_value.dyn_into().ok()?;
            
            if !response.ok() {
                return None;
            }

            let array_buffer = wasm_bindgen_futures::JsFuture::from(response.array_buffer().ok()?).await.ok()?;
            let u8_array = js_sys::Uint8Array::new(&array_buffer);
            let bytes = u8_array.to_vec();

            if bytes.is_empty() {
                return None;
            }
            Some(bytes)
        }
    }.await {
        Some(bytes) => bytes,
        None => {
            web_sys::console::log_1(&"Failed to fetch DB bytes".into());
            return Vec::new();
        }
    };

    // Save to OPFS
    let dir = match app_specific_dir().await {
        Ok(d) => d,
        Err(e) => {
            web_sys::console::log_1(&format!("Failed to access OPFS dir: {:?}", e).into());
            return Vec::new();
        }
    };

    match dir
        .get_file_handle_with_options("chronik.sqlite", &GetFileHandleOptions { create: true })
        .await
    {
        Ok(mut file) => {
            let write_options = CreateWritableOptions {
                keep_existing_data: false,
            };
            match file.create_writable_with_options(&write_options).await {
                Ok(mut writer) => {
                    if writer.write_at_cursor_pos(db_bytes.clone()).await.is_ok() {
                        let _ = writer.close().await;
                        web_sys::console::log_1(&"Saved DB to OPFS".into());
                        return db_bytes;
                    }
                }
                Err(e) => {
                    web_sys::console::log_1(
                        &format!("Failed to create writable stream: {:?}", e).into(),
                    );
                }
            }
        }
        Err(e) => {
            web_sys::console::log_1(&format!("Failed to get file handle: {:?}", e).into());
        }
    }

    Vec::new()
}
