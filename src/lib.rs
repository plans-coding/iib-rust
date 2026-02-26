use wasm_bindgen::prelude::*;
use serde_json::json;
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use serde_json::Value;
use wasm_bindgen_futures::future_to_promise;
use js_sys::Promise;
use web_sys::{window,HtmlElement};
use chrono::Local;

mod filecontent;
mod sqlite_query;

macro_rules! define_resources {
    ($($name:ident => $path:expr),+ $(,)?) => {
        $( pub const $name: &str = include_str!($path); )+
        pub const ALL_QUERIES: &[(&str, &str)] = &[ $( define_resources!(@is_query $name) ),+ ];
    };
    (@is_query $name:ident) => { { (stringify!($name), $name) } };
}

define_resources! {

    //APP_LOGIC => "../src/app_logic.json",
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

}

static DB_BYTES: OnceCell<Vec<u8>> = OnceCell::new();
static RENDER_STRUCTURE: OnceCell<Mutex<Value>> = OnceCell::new();

// Other files
//static CHART_JS: &str = include_str!("../bundle/chartjs/chart.js");
//static MAPLIBRE_JS: &str = include_str!("../bundle/maplibre-gl/maplibre-gl.js");
//static MAPLIBRE_CSS: &str = include_str!("../bundle/maplibre-gl/maplibre-gl.css");
//static BEWGUNG_CSS: &str = include_str!("../bewegung.css");

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
    fn load_code_editor();
    fn initiate_spreadsheet();
    fn custom_queries();
    //fn inject_css(css: &str);
    // Other
    fn initialize_theme_color();

    fn check_immich_authorization();
    fn init_create_trip();

    fn load_filter_OPFS();

    #[wasm_bindgen(catch)]
    async fn get_filter_value_OPFS() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn check_available_update() -> Result<JsValue, JsValue>;
}

// -----------------------------------------------------------------------
// REAL WASM START
// -----------------------------------------------------------------------
#[wasm_bindgen(start)]
fn start() {

    wasm_bindgen_futures::spawn_local(async {
    
        let (db_bytes, render_structure) = session_load().await;

        DB_BYTES.set(db_bytes).expect("DB already initialized");
        RENDER_STRUCTURE
            .set(Mutex::new(render_structure))
            .expect("Render structure already initialized");

        page_load_internal().await;
    });
}

// -----------------------------------------------------------------------
// MAKE RUST FUNCTIONS AVAILABLE FOR JAVASCRIPT
// -----------------------------------------------------------------------
#[wasm_bindgen]
pub fn page_load() {
    wasm_bindgen_futures::spawn_local(async {
        page_load_internal().await;
    });
}
#[wasm_bindgen]
pub fn chart_js() -> String {
    CHART_JS.to_string()
}
/*#[wasm_bindgen]
pub fn maplibre_js() -> String {
    MAPLIBRE_JS.to_string()
}*/

#[wasm_bindgen]
pub fn get_predefined_query(name: &str) -> Option<String> {
    ALL_QUERIES
    .iter()
    .find(|(k, _)| *k == name)
    .map(|(_, v)| v.to_string())
}

#[wasm_bindgen]
pub fn user_run_sql(sql: String) -> Promise {
    // Wrap your async Rust code in a JS Promise
    future_to_promise(async move {
        sqlite_query::user_run_sql_internal(sql).await;
        // Return undefined (JS will see it as resolved)
        Ok(JsValue::undefined())
    })
}


// -----------------------------------------------------------------------
// INITIATE SESSION
// -----------------------------------------------------------------------

    async fn session_load() -> (Vec<u8>, serde_json::Value) {
        let db_bytes = filecontent::get_sqlite_binary().await;
        if !db_bytes.is_empty() { sqlite_query::init_db(&db_bytes); }

        let path = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok())
        .and_then(|p| p.get("path"))
        .unwrap_or_else(|| "explore".to_string());

        let mut results = sqlite_query::get_query_data(&db_bytes, vec![
            ("file".into(), "SELECT Value FROM bewx_Settings WHERE Attribute = 'LanguageFile' LIMIT 1;".into()),
            ("settings".into(), "SELECT * FROM bewx_Settings;".into()),
            ("common_trip_domains".into(), QUERY_COMMON_TRIP_DOMAINS.into()),
            ("common_participant_groups".into(), QUERY_COMMON_PARTICIPANT_GROUPS.into()),
            ("common_trip_labels".into(), QUERY_COMMON_TRIP_LABELS.into()),
        ]).await;

        let translation = match results.pointer("/file/0/Value").and_then(|v| v.as_str()) {
            Some(f) => filecontent::fetch_json(&format!("static/languages/{}", f)).await.unwrap_or(json!({})),
            None => json!({}),
        };

        let mut settings = json!({});
        if let Some(arr) = results["settings"].as_array() {
            for s in arr {
                let group = s["AttributeGroup"].as_str().unwrap_or("General");
                settings.as_object_mut().unwrap()
                .entry(group).or_insert(json!({}))
                .as_object_mut().unwrap()
                .insert(s["Attribute"].as_str().unwrap_or("Unknown").to_string(), s["Value"].clone());
            }
        }

        if let Some(obj) = results.as_object_mut() {
            obj.remove("settings");
            obj.remove("file");
        }

        let global_data = json!({ "path": path, "translation": translation, "settings": settings, "common": results });

        if let Err(e) = render2dom(TEMPLATE_MENU, &global_data, "menu", false) {
            let msg = format!("render2dom failed: {}", e);
            web_sys::console::log_1(&msg.clone().into());
            let _ = window().and_then(|w| w.document()).and_then(|d| d.get_element_by_id("error_msg"))
            .and_then(|el| el.dyn_into::<HtmlElement>().ok())
            .map(|html| html.set_inner_text(&msg));
        } else if let Ok(content) = render2dom(TEMPLATE_MENU, &global_data, "menu", false) {
            web_sys::console::log_1(&format!("render2dom succeeded, length: {}", content.len()).into());
        }

        initialize_theme_color();
        (db_bytes, global_data)
    }


// -----------------------------------------------------------------------
// HOT RELOAD
// -----------------------------------------------------------------------
async fn page_load_internal() {


    //web_sys::console::log_1(&">>----------------------".into());

    let db_bytes = DB_BYTES.get().expect("DB not initialized");
    let render_structure_mutex = RENDER_STRUCTURE.get().expect("Render structure missing");
    let mut map_request = "";

    // Lock the Mutex to get a mutable reference
    let mut render_structure = render_structure_mutex.lock().expect("ERROR");

    let path = web_sys::window().expect("No window available").location().search().ok()
    .as_deref().and_then(|s| web_sys::UrlSearchParams::new_with_str(s).ok()).and_then(|params| params.get("path"));

    let page = path.as_deref().unwrap_or("explore");
   
    //web_sys::console::log_1(&format!("Loading page: {}",page).into());
   
    render_structure["all"]["query_params"]["path"] = path.clone().into();

    // READ APPLIED FILTERS  -----------------------------------------------------------------------

    let filter_values = get_filter_value_OPFS().await.unwrap();
    render_structure["all"]["filters"] = serde_wasm_bindgen::from_value(filter_values).unwrap();

    // Prepare filters
    
    let participant_group = if render_structure["all"]["filters"]["participantGroup"].as_array().map_or(true, |a| a.is_empty()) {
        "(ParticipantGroup)".to_string()
    } else {
        format!("({})", render_structure["all"]["filters"]["participantGroup"].as_array().expect("ERROR").iter().filter_map(|v| v.as_str()).map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(","))
    };
    let trip_domain = if render_structure["all"]["filters"]["tripDomain"].as_array().map_or(true, |a| a.is_empty()) {
        "(TripDomain)".to_string()
    } else {
        format!("({})", render_structure["all"]["filters"]["tripDomain"].as_array().expect("ERROR").iter().filter_map(|v| v.as_str()).map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(","))
    };
    let trip_label = if render_structure["all"]["filters"]["tripLabel"]
        .as_array()
        .map_or(true, |a| a.is_empty())
    {
        // no filter → tautology
        "1=1".to_string()
    } else {
        // build AND chain of LIKEs for all labels
        let conds = render_structure["all"]["filters"]["tripLabel"]
            .as_array()
            .expect("ERROR")
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| format!("(',' || REPLACE(TripLabels,' ','') || ',') LIKE '%,{},%'", s.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(" AND "); // use "AND" if you want ALL labels present; "OR" if any label suffices

        format!("({})", conds)
    };

    //web_sys::console::log_1(&serde_json::to_string(&render_structure["all"]["filters"]).expect("ERROR").into());

    use std::collections::HashMap;
    let cover_photos_list_opt = filecontent::cover_photos_list_from_opfs().await;
    let cover_photos_map: HashMap<String, String> = match cover_photos_list_opt {
        Some(json_str) => serde_json::from_str(&json_str).expect("Invalid JSON"),
        None => HashMap::new(),
    };


    // -----------------------------------------------------------------------
    // Fourth: Page specific data
    // -----------------------------------------------------------------------
        match page {
            "explore" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/explore/title").and_then(|v| v.as_str()).unwrap_or("Explore"),
                    "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_EXPLORE),
                    "queries": [
                        ["explore", QUERY_EXPLORE
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)],
                    ]});
                render_structure["all"]["cover_photos_list"] = serde_json::to_value(&cover_photos_map).expect("Failed to convert map to Value");
            }
            "overview:year" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/overview/year").and_then(|v| v.as_str()).unwrap_or("Overview: Year"),
                    "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_OVERVIEW_YEAR),
                    "queries": [
                        ["overviewYear", QUERY_OVERVIEW_YEAR.replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)]
                    ]});
            }
            "overview:country" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/overview/country").and_then(|v| v.as_str()).unwrap_or("Overview: Country"),
                    "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_OVERVIEW_COUNTRY),
                    "queries": [
                         // Replace "c.Continent = 'Europa'" in QUERY_OVERVIEW_COUNTRY with value from settings in future version
                         ["overviewCountry", QUERY_OVERVIEW_COUNTRY.to_string().replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)]
                     ]});
            }
            "overview:plain" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/overview/plain").and_then(|v| v.as_str()).unwrap_or("Overview: Plain"),
                    "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_OVERVIEW_PLAIN),
                    "queries": [
                        ["overviewPlain", QUERY_OVERVIEW_PLAIN.to_string().replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)],
                         // Replace "c.Continent = 'Europa'" in QUERY_OVERVIEW_COUNTRY with value from settings in future version
                        /*["overviewCountry", QUERY_OVERVIEW_COUNTRY.to_string().replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)]*/
                    ]});
            }
            "map" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/map/title").and_then(|v| v.as_str()).unwrap_or("Map"),
                    "template": TEMPLATE_MAP,
                    "queries": [
                        ["map_country_list", QUERY_MAP_COUNTRY_LIST.replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)],
                        ["map_data", QUERY_MAP_CONTOUR.replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)], //contour
                        ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS.to_string()],
                    ]});
                map_request = "contour";
                // See later in code for special cases
            }
            "statistics:summary" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/statistics/summary").and_then(|v| v.as_str()).unwrap_or("Statistics: Summary"),
                    "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_STATISTICS_SUMMARY),
                    "queries": [
                        ["statistics_visits", QUERY_STATISTICS_VISITS.replace("SELECT\n    Country,\n    OL,\n    SS,\n    VSS,\n    PS,\n    OLMQ,\n    SSMQ,\n    VSSMQ,\n    PSMQ\nFROM Aggregated\nORDER BY OL DESC;", "SELECT COUNT(DISTINCT Country) AS TripCount FROM Aggregated;").replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)],
                        ["statistics_trip_count", QUERY_STATISTICS_TRIP_COUNT.replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)],
                        ["statistics_per_domain_year", QUERY_STATISTICS_PER_DOMAIN_YEAR.replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)],
                        ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS.to_string()],
                    ]});
            }
            "statistics:visits" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/statistics/visits").and_then(|v| v.as_str()).unwrap_or("Statistics: Visits"),
                    "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_STATISTICS_VISITS),
                    "queries": [
                        ["statistics_visits", QUERY_STATISTICS_VISITS.replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)]
                    ]});
            }
            "statistics:overnights" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/statistics/overnights").and_then(|v| v.as_str()).unwrap_or("Statistics: Overnights"),
                    "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_STATISTICS_OVERNIGHTS),
                    "queries": [
                        ["statistics_overnights", QUERY_STATISTICS_OVERNIGHTS.replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)],
                    ]});
            }
            "statistics:themes" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/settings/Plugin/Theme/translation").and_then(|v| v.as_str()).unwrap_or("Themes"),
                    "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_STATISTICS_THEMES),
                    "queries": [
                        ["statistics_theme_count", QUERY_STATISTICS_THEME_COUNT.replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)
                        .replace("1=1", &trip_label)]
                    ]});
            }
            "dataset" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/dataset/title").and_then(|v| v.as_str()).unwrap_or("Dataset"),
                    "settings": render_structure["all"]["settings"],
                    "template": TEMPLATE_DATASET,
                    "queries": [
                        ["table_list", "SELECT name FROM sqlite_master WHERE type IN ('table', 'view') ORDER BY name;"],
                        ["stored_custom_queries", "SELECT ROWID, Name FROM com_CodeCollection WHERE Target = 'BewDataset';"],
                    ]});
                render_structure["all"]["query_templates"] = json!(ALL_QUERIES.iter().map(|(name, _)| name).collect::<Vec<_>>());
            }
            "source" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/source/title").and_then(|v| v.as_str()).unwrap_or("Source"),
                    "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_SOURCE),
                    "queries": [
                        ["cover_photo_original_paths", "SELECT OuterId, CoverPhoto FROM bewa_Overview WHERE CoverPhoto IS NOT NULL;"],
                    ]});
                render_structure["all"]["db_loaded"] = json!(if !&db_bytes.is_empty() { "stored" } else { "missing" });
            }
            "about" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/about/title").and_then(|v| v.as_str()).unwrap_or("About"),
                    "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_ABOUT),
                    });
                let update_info = check_available_update()
                    .await
                    .ok()
                    .and_then(|v| v.as_string())
                    .unwrap_or_default();
                render_structure["all"]["update_info"] = update_info.into();
                render_structure["all"]["current_version"] = json!(CURRENT_VERSION);
            }
            "report" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/toolbox/report").and_then(|v| v.as_str()).unwrap_or("Report"),
                        "template": format!("{}{}", TEMPLATE_BREADCRUMBS.replace("_PAGE_", page), TEMPLATE_TOOLBOX_REPORT),
                        });
                render_structure["all"]["date_now"] = json!(Local::now().format("%Y-%m-%d").to_string());
                render_structure["all"]["year_now"] = json!(Local::now().format("%Y").to_string());
            }
            "input" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/toolbox/input").and_then(|v| v.as_str()).unwrap_or("Input"),
                        "template": TEMPLATE_TOOLBOX_INPUT,
                    });
                }
            _ => {
                
                //web_sys::console::log_1(&"Second tier.".into());

                if let Some(rest) = page.strip_prefix("trip:") {
                    let mut parts = rest.splitn(2, ':');

                    let outer_id = parts.next().filter(|s| !s.is_empty());
                    let inner_id = parts.next().filter(|s| !s.is_empty());

                    match (outer_id, inner_id) {
                        // trip::yyy  → only inner
                        // TO DO: rebuild to redirect to corresponding outerid
                        (None, Some(inner_id)) => {
                            render_structure["page"] = json!({
                                "title": inner_id,
                                "template": TEMPLATE_TRIP,
                                "queries": [
                                    ["trip_summary", QUERY_TRIP_SUMMARY.replace("= InnerId", &format!("= '{}'", inner_id))],
                                    ["trip_events", QUERY_TRIP_EVENTS.replace("= e.InnerId", &format!("= '{}'", inner_id))],
//                                    ["trip_all_trips", QUERY_TRIP_ALL_TRIPS],
                                    ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS],
                                    // Lägg till filter
	                            ["trip_unique_countries", QUERY_TRIP_UNIQUE_COUNTRIES.replace("= InnerId", &format!("= '{}'", inner_id))],
                                    ["trip_border_crossings", QUERY_TRIP_BORDER_CROSSINGS.replace("= a.InnerId", &format!("= '{}'", inner_id))],
                                    ["trip_map_pins_overall", QUERY_TRIP_MAP_PINS_OVERALL.replace("= InnerId", &format!("= '{}'", inner_id))],
                                    ["trip_map_pins_accommodation", QUERY_TRIP_MAP_PINS_ACCOMMODATION.replace("= o.InnerId", &format!("= '{}'", inner_id))],
                                    ["trip_previous", QUERY_TRIP_PREVIOUS.replace("= InnerId", &format!("= '{}'", inner_id))
                                    .replace("(ParticipantGroup)", &participant_group)
                                    .replace("(TripDomain)", &trip_domain)
                                    .replace("1=1", &trip_label)],
                                    ["trip_next", QUERY_TRIP_NEXT.replace("= InnerId", &format!("= '{}'", inner_id))
                                    .replace("(ParticipantGroup)", &participant_group)
                                    .replace("(TripDomain)", &trip_domain)
                                    .replace("1=1", &trip_label)],
                                    ["trip_immich_desc_search", QUERY_TRIP_IMMICH_DESC_SEARCH.replace("= InnerId", &format!("= '{}'", inner_id))],
                                    ["trip_immich_album_name", QUERY_TRIP_IMMICH_ALBUM_NAME.replace("= InnerId", &format!("= '{}'", inner_id))],
                                ]});
                            render_structure["all"]["cover_photos_list"] = serde_json::to_value(&cover_photos_map).expect("Failed to convert map to Value");
                            map_request = "trip";
                        }

                        // trip:xxx or trip:xxx:yyy → outer exists
                        (Some(outer_id), _) => {

                        // Title med outer id + dagbok + pass
                        render_structure["page"] = json!({
                            "title": outer_id,
                            "template": TEMPLATE_TRIP,
                            "queries": [
                                ["trip_summary", QUERY_TRIP_SUMMARY.to_string().replace("= OuterId", &format!("= '{}'", outer_id))],
                                ["trip_events", QUERY_TRIP_EVENTS.to_string().replace("= o.OuterId", &format!("= '{}'", outer_id))],
//                                ["trip_all_trips", QUERY_TRIP_ALL_TRIPS.to_string()],
                                ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS.to_string()],
                                // Lägg till filter
                                ["trip_unique_countries", QUERY_TRIP_UNIQUE_COUNTRIES.replace("= OuterId", &format!("= '{}'", outer_id))],
                                ["trip_border_crossings", QUERY_TRIP_BORDER_CROSSINGS.replace("= b.OuterId", &format!("= '{}'", outer_id))],
                                ["trip_map_pins_overall", QUERY_TRIP_MAP_PINS_OVERALL.replace("= OuterId", &format!("= '{}'", outer_id))],
                                ["trip_map_pins_accommodation", QUERY_TRIP_MAP_PINS_ACCOMMODATION.replace("= o.OuterId", &format!("= '{}'", outer_id))],
                                ["trip_previous", QUERY_TRIP_PREVIOUS.replace("= OuterId", &format!("= '{}'", outer_id))
                                    .replace("(ParticipantGroup)", &participant_group)
                                    .replace("(TripDomain)", &trip_domain)
                                    .replace("1=1", &trip_label)],
                                ["trip_next", QUERY_TRIP_NEXT.replace("= OuterId", &format!("= '{}'", outer_id))
                                    .replace("(ParticipantGroup)", &participant_group)
                                    .replace("(TripDomain)", &trip_domain)
                                    .replace("1=1", &trip_label)],
                                ["trip_immich_desc_search", QUERY_TRIP_IMMICH_DESC_SEARCH.replace("= OuterId", &format!("= '{}'", outer_id))],
                                ["trip_immich_album_name", QUERY_TRIP_IMMICH_ALBUM_NAME.replace("= OuterId", &format!("= '{}'", outer_id))],
                                ["trip_extension_movie", QUERY_TRIP_EXTENSION_MOVIE.replace("_OUTER_ID_", &outer_id)],
                            ]});
                            render_structure["all"]["cover_photos_list"] = serde_json::to_value(&cover_photos_map).expect("Failed to convert map to Value");
                            map_request = "trip";

                        }
                        _ => {}
                    }
                }

                if let Some(suffix) = page.strip_prefix("images:") {

                    let mut parts = suffix.splitn(2, ':');

                    if let (Some(trip_id), Some(trip_date)) = (parts.next(), parts.next()) {
                        let trip_id = trip_id.to_string();
                        let trip_date = trip_date.to_string();

                        render_structure["page"] = json!({
                            "title": suffix,
                            "template": TEMPLATE_IMAGES,
                            "queries": [
                                ["images_date_list", QUERY_IMAGES_DATE_LIST.replace("/*_OUTER_ID_*/",&trip_id)],
                                ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS.to_string()],
                                ["images_photo_time", QUERY_IMAGES_PHOTO_TIME.replace("/*_OUTER_ID_*/",&trip_id)],
                        ]});
                        render_structure["all"]["trip_date"] = json!(trip_date);
                        render_structure["all"]["trip_id"] = json!(trip_id);
                    }
                }
                
                if let Some(suffix) = page.strip_prefix("map:") {
                
                    if let Some(country) = suffix.strip_prefix("country:") {
                    
                        // Title med outer id + dagbok + pass
                        render_structure["page"] = json!({
                            "title": render_structure.pointer("/all/translation/map/title").and_then(|v| v.as_str()).unwrap_or("Map"),
                            "template": TEMPLATE_MAP,
                            "queries": [
                                ["map_country_list", QUERY_MAP_COUNTRY_LIST.replace("(ParticipantGroup)", &participant_group)
                                .replace("(TripDomain)", &trip_domain)
                                .replace("1=1", &trip_label)],
                                ["map_data", QUERY_MAP_COUNTRY.replace("_COUNTRY_",country).replace("(ParticipantGroup)", &participant_group)
                                .replace("(TripDomain)", &trip_domain)
                                .replace("1=1", &trip_label)], //country
                                ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS.to_string()],
                            ]});
                        map_request = "country";
                    
                    } else if let Some(theme) = suffix.strip_prefix("theme:") {
                    
                        // Title med outer id + dagbok + pass
                        render_structure["page"] = json!({
                            "title": render_structure.pointer("/all/translation/map/title").and_then(|v| v.as_str()).unwrap_or("Map"),
                            "template": TEMPLATE_MAP,
                            "queries": [
                                ["map_country_list", QUERY_MAP_COUNTRY_LIST.replace("(ParticipantGroup)", &participant_group)
                                .replace("(TripDomain)", &trip_domain)
                                .replace("1=1", &trip_label)],
                                ["map_data", QUERY_MAP_THEME.replace("_THEME_",theme).replace("(ParticipantGroup)", &participant_group)
                                .replace("(TripDomain)", &trip_domain)
                                .replace("1=1", &trip_label)], //theme
                                ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS.to_string()],
                            ]});
                        map_request = "theme";
                    
                    }

                }
                
                if let Some(suffix) = page.strip_prefix("search:") {
                    // Title med outer id + dagbok + pass
                    render_structure["page"] = json!({
                        "title": suffix,
                        "template": TEMPLATE_SEARCH,
                        "settings": serde_json::to_value(&render_structure["all"]["settings"]).expect("ERROR"),
                        "queries": [
                            ["search_trip", QUERY_SEARCH_TRIP.to_string().replace("/*_STRING_*/", suffix).replace("(ParticipantGroup)", &participant_group)
                            .replace("(TripDomain)", &trip_domain)
                            .replace("1=1", &trip_label)],
                            ["search_event", QUERY_SEARCH_EVENT.to_string().replace("/*_STRING_*/", suffix).replace("(ParticipantGroup)", &participant_group)
                            .replace("(TripDomain)", &trip_domain)
                            .replace("1=1", &trip_label)],
                        ]});
                }

                if let Some(suffix) = page.strip_prefix("report:output:") {

                    let mut parts = suffix.splitn(2, ':');

                    if let (Some(title_string), Some(backside_string)) = (parts.next(), parts.next()) {
                        let title_string = title_string.to_string();
                        let backside_string = backside_string.to_string();

                        render_structure["page"] = json!({
                            "title": suffix,
                            "template": TEMPLATE_TOOLBOX_REPORT_OUTPUT,
                            "queries": [
                                ["all_overview", QUERY_REPORT_ALL_OVERVIEW.replace("(ParticipantGroup)", &participant_group)
                                .replace("(TripDomain)", &trip_domain)
                                .replace("1=1", &trip_label)],
                                ["all_events", QUERY_REPORT_ALL_EVENTS.replace("(ParticipantGroup)", &participant_group)
                                .replace("(TripDomain)", &trip_domain)
                                .replace("1=1", &trip_label)],
                            ]});
                        render_structure["all"]["title_string"] = json!(title_string);
                        render_structure["all"]["backside_string"] = json!(backside_string);
                    }
                }
                
            }
        }

    // -----------------------------------------------------------------------
    // Fifth: Render content
    // -----------------------------------------------------------------------

        //web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]).expect("ERROR").into());
        
        // SET TITLE  -----------------------------------------------------------------------
    
        //web_sys::console::log_1(&"----------------------".into());

        let title = render_structure["page"]["title"].as_str().unwrap_or("Default Title");
        web_sys::window().expect("ERROR").document().expect("ERROR").set_title(&format!("{title} - Immer in Bewegung"));

        //web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["title"]).expect("ERROR").into());
    
        // RUN SQLITE QUERIES  -----------------------------------------------------------------------

        //web_sys::console::log_1(&"----------------------".into());
        let combined_query: Vec<(String, String)> = render_structure["page"]["queries"]
        .as_array().unwrap_or(&Vec::new()).iter().map(|row| {
            // Each row: [key, value]
            let k = row[0].as_str().unwrap_or("").to_string();
            let v = row[1].as_str().unwrap_or("").to_string();
            (k, v)
        })
        .collect();

        let query_response: serde_json::Value = sqlite_query::get_query_data(&db_bytes, combined_query).await;
        //web_sys::console::log_1(&serde_json::to_string(&query_response).expect("ERROR").into());
    
        let mut merged_structure = render_structure.clone();

        // Merge if both are objects
        match (&mut merged_structure, query_response) {
            (serde_json::Value::Object(target), serde_json::Value::Object(source)) => {
                for (k, v) in source {
                    target.insert(k, v);
                }
            }
            (_, other) => {
                merged_structure = other;
            }
        }

        // RENDER TO 'APP'  -----------------------------------------------------------------------
        //let _ = render::render2dom(&render_structure["page"]["template"].as_str().expect("template must be a string"), &merged_structure, "app", true);

        let translate_iib_markdown_2link =!matches!(map_request, "trip" | "contour" | "country" | "theme");

        web_sys::console::log_1(&"----------------------HEJ".into());
        web_sys::console::log_1(&serde_json::to_string(&merged_structure).expect("ERROR").into());

        let rendered_result = render2dom(
            &render_structure["page"]["template"]
            .as_str()
            .expect("template must be a string"),
                &merged_structure,
                "app",
                translate_iib_markdown_2link,
        );

        match &rendered_result {
            Ok(content) => web_sys::console::log_1(&JsValue::from_str(&format!(
                "render2dom succeeded, content length: {}",
                content.len()
            ))),
            /*Err(e) => web_sys::console::log_1(&JsValue::from_str(&format!(
                "render2dom failed: {}",
                e
            ))),*/
            Err(e) => {
                let msg = format!("render2dom failed: {}", e);

                web_sys::console::log_1(&JsValue::from_str(&msg));

                if let Some(document) = window().and_then(|w| w.document()) {
                    if let Some(el) = document.get_element_by_id("error_msg") {
                        if let Ok(html) = el.dyn_into::<HtmlElement>() {
                            html.set_inner_text(&msg); // safer than inner_html
                        }
                    }
                }
            }
        }

        let _ = rendered_result;

        load_filter_OPFS();
        
        // POST CODE  -----------------------------------------------------------------------

        match map_request {
            "trip" => load_trip_map(),
            "contour" => load_contour_map(),
            "country" => load_country_map(),
            "theme" => load_theme_map(),
            _ => {}
        }

        match page {
           "dataset" => {
                load_code_editor();
                initiate_spreadsheet();
                //custom_queries(); // Need input value (e.g. get code editor content)
            }
            "statistics:summary" => {
                initializeChart();
            }
            "statistics:overnights" => {
                initializeChartOvernights();
            }
            "source" => check_immich_authorization(),
            "input" => init_create_trip(),
            "about" => {},
            _ => {}
        }

}

use tera::{Context, Tera};
use regex::Regex;

fn render2dom(
    template_content: &str,
    json_object: &serde_json::Value, // Use Value directly
    element_id: &str,
    include_wrapper: bool,
) -> Result<String, String> {

    let context = Context::from_serialize(json_object)
    .map_err(|e| format!("Context error: {e}"))?;

    let mut rendered = Tera::one_off(template_content, &context, true)
    .map_err(|e| format!("Render error: {e}"))?;

    // Convert text to link if requested
    if include_wrapper {
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
    }

    web_sys::window()
    .and_then(|w| w.document())
    .and_then(|d| d.get_element_by_id(element_id))
    .ok_or_else(|| format!("Element #{} not found", element_id))?
    .set_inner_html(&rendered);

    Ok(rendered)
}
