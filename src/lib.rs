use wasm_bindgen::prelude::*;
use serde_json::json;
use helper::SqlFilterReplace;
use filecontent::load_filter_from_opfs;

mod filecontent;
mod sqlite_query;
mod render;
mod query_params;
mod helper;

// Templates
const TEMPLATE_MENU: &str = include_str!("../templates/_menu.tera");
const TEMPLATE_EXPLORE: &str = include_str!("../templates/explore.tera");
const TEMPLATE_OVERVIEW_YEAR: &str = include_str!("../templates/overview_year.tera");
const TEMPLATE_OVERVIEW_COUNTRY: &str = include_str!("../templates/overview_country.tera");
const TEMPLATE_OVERVIEW_PLAIN: &str = include_str!("../templates/overview_plain.tera");
const TEMPLATE_TRIP: &str = include_str!("../templates/trip.tera");
const TEMPLATE_IMAGES: &str = include_str!("../templates/images.tera");
const TEMPLATE_MAP: &str = include_str!("../templates/map.tera");
const TEMPLATE_SEARCH: &str = include_str!("../templates/search.tera");
const TEMPLATE_STATISTICS_SUMMARY: &str = include_str!("../templates/statistics_summary.tera");
const TEMPLATE_STATISTICS_VISITS: &str = include_str!("../templates/statistics_visits.tera");
const TEMPLATE_STATISTICS_OVERNIGHTS: &str = include_str!("../templates/statistics_overnights.tera");
const TEMPLATE_STATISTICS_THEMES: &str = include_str!("../templates/statistics_themes.tera");
const TEMPLATE_DATASET: &str = include_str!("../templates/dataset.tera");
const TEMPLATE_ABOUT: &str = include_str!("../templates/about.tera");
const TEMPLATE_SOURCE: &str = include_str!("../templates/source.tera");

// Advanced queries
const QUERY_EXPLORE: &str = include_str!("../queries/explore.sql");
const QUERY_OVERVIEW_YEAR: &str = include_str!("../queries/overview_year.sql");
const QUERY_OVERVIEW_COUNTRY: &str = include_str!("../queries/overview_country.sql");
const QUERY_TRIP_BORDER_CROSSINGS: &str = include_str!("../queries/trip_border_crossings.sql");
const QUERY_STATISTICS_VISITS: &str = include_str!("../queries/statistics_visits.sql");
const QUERY_STATISTICS_OVERNIGHTS: &str = include_str!("../queries/statistics_overnights.sql");
const QUERY_STATISTICS_PER_DOMAIN_YEAR: &str = include_str!("../queries/statistics_per_domain_year.sql");
const QUERY_STATISTICS_THEME_COUNT: &str = include_str!("../queries/statistics_theme_count.sql");
const QUERY_TRIP_MAP_PINS: &str = include_str!("../queries/trip_map_pins.sql");

// Simple queries
const QUERY_COMMON_PARTICIPANT_GROUPS: &str = include_str!("../queries/simple/common_participant_groups.sql");
const QUERY_COMMON_TRIP_DOMAINS: &str = include_str!("../queries/simple/common_trip_domains.sql");
const QUERY_IMAGES_DATE_LIST: &str = include_str!("../queries/simple/images_date_list.sql");
const QUERY_IMAGES_PHOTO_TIME: &str = include_str!("../queries/simple/images_photo_time.sql");
const QUERY_MAP_CONTOUR: &str = include_str!("../queries/simple/map_contour.sql");
const QUERY_MAP_COUNTRY: &str = include_str!("../queries/simple/map_country.sql");
const QUERY_MAP_COUNTRY_LIST: &str = include_str!("../queries/simple/map_country_list.sql");
const QUERY_MAP_THEME: &str = include_str!("../queries/simple/map_theme.sql");
const QUERY_SEARCH_EVENT: &str = include_str!("../queries/simple/search_event.sql");
const QUERY_SEARCH_TRIP: &str = include_str!("../queries/simple/search_trip.sql");
const QUERY_STATISTICS_TRIP_COUNT: &str = include_str!("../queries/simple/statistics_trip_count.sql");
const QUERY_TRIP_ALL_TRIPS: &str = include_str!("../queries/simple/trip_all_trips.sql");
const QUERY_TRIP_EVENTS: &str = include_str!("../queries/simple/trip_events.sql");
const QUERY_TRIP_SUMMARY: &str = include_str!("../queries/simple/trip_summary.sql");
const QUERY_TRIP_PREVIOUS: &str = include_str!("../queries/simple/trip_previous.sql");
const QUERY_TRIP_NEXT: &str = include_str!("../queries/simple/trip_next.sql");


#[wasm_bindgen(start)]
fn start() {

    wasm_bindgen_futures::spawn_local(async {

    // -----------------------------------------------------------------------
    // First: Get sqlite database binary
    // -----------------------------------------------------------------------

        let db_bytes = filecontent::get_sqlite_binary().await;
        if !db_bytes.is_empty() {
            web_sys::console::log_1(&format!("DB size: {}", db_bytes.len()).into());
        } else {
            web_sys::console::log_1(&"No DB loaded.".into());
            // Set query parameter 'p' to 'source'
            query_params::set_query_params(&json!({"path":"more:source"}));
            // set page = configure -- needed?
        }

    // -----------------------------------------------------------------------
    // Second: Handle query parameters
    // -----------------------------------------------------------------------

        let query_params = query_params::get_query_params();
        // Presume ?p=overview if not set at all
        let page = match &query_params["path"] { serde_json::Value::String(s) if !s.is_empty() => s.as_str(), _ => "explore", };
        web_sys::console::log_1(&format!("Loading page: {}",page).into());

    // -----------------------------------------------------------------------
    // Third: Common data for all pages
    // -----------------------------------------------------------------------

        let mut render_structure = json!({});
        render_structure["all"]["query_params"] = query_params.clone();

        // If "path" is missing or empty, set it to "overview"
        let p_is_empty = render_structure["all"]["query_params"]["path"].as_str().map(|s| s.is_empty()).unwrap_or(true);
        if p_is_empty { render_structure["all"]["query_params"]["path"] = json!("explore"); }

        render_structure["all"]["time"] = helper::build_time_json();
        web_sys::console::log_1(&serde_json::to_string(&render_structure["all"]).unwrap().into());

        // Get translation
        let translation_query = vec![
            ("translation_filename".to_string(), "SELECT Value FROM bewxx_Settings WHERE AttributeGroup = 'Base' AND Attribute = 'LanguageFile';".to_string())
        ];
        let translation_filename = sqlite_query::get_query_data(&db_bytes, translation_query).await;
        let json_obj: serde_json::Value = serde_json::to_value(&translation_filename).unwrap();
        let translation_filename_extracted = format!("languages/{}",json_obj["translation_filename"][0]["Value"].as_str().expect("Expected settings[0].Value to be a string"));
        web_sys::console::log_1(&translation_filename_extracted.as_str().into());
        let translation_content = filecontent::fetch_json(&translation_filename_extracted).await.unwrap_or(serde_json::Value::String("".to_string()));;
        //web_sys::console::log_1(&serde_json::to_string(&translation_content).unwrap().into());

        // Get all settings
        let settings_query = vec![
            ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string())
        ];
        let settings_response = sqlite_query::get_query_data(&db_bytes, settings_query).await;


        //crender_structure["all"]["settings"] = serde_json::to_value(&settings_response["settings"]).unwrap();
        render_structure["all"]["settings"] = helper::transform_settings(&settings_response["settings"].as_array().unwrap());
        render_structure["all"]["translation"] = translation_content;//.expect("Error with translation data.");
        web_sys::console::log_1(&serde_json::to_string(&render_structure["all"]["settings"]).unwrap().into());
        
        
        // RENDER TO 'MENU'  -----------------------------------------------------------------------
        let common_data = vec![
            ("common_trip_domains".to_string(), QUERY_COMMON_TRIP_DOMAINS.to_string()),
            ("common_participant_groups".to_string(), QUERY_COMMON_PARTICIPANT_GROUPS.to_string())
        ];
        render_structure["all"]["common"] = sqlite_query::get_query_data(&db_bytes, common_data).await;
        let _ = render::render2dom(TEMPLATE_MENU, &render_structure["all"], "menu");
        
        // READ APPLIED FILTERS  -----------------------------------------------------------------------
        
        let filters_value = filecontent::load_filter_from_opfs()
        .await
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .unwrap_or(serde_json::Value::Null);
    
        web_sys::console::log_1(&"----------------------".into());
        render_structure["all"]["filters"] = filters_value;
        web_sys::console::log_1(&serde_json::to_string(&render_structure["all"]["filters"]).unwrap().into());
        
        // Prepare filters
        let participant_group = if render_structure["all"]["filters"]["ParticipantGroup"].as_array().map_or(true, |a| a.is_empty()) {
            "(ParticipantGroup)".to_string()
        } else {
            format!("({})", render_structure["all"]["filters"]["ParticipantGroup"].as_array().unwrap().iter().filter_map(|v| v.as_str()).map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(","))
        };
        let trip_domain = if render_structure["all"]["filters"]["TripDomain"].as_array().map_or(true, |a| a.is_empty()) {
            "(TripDomain)".to_string()
        } else {
            format!("({})", render_structure["all"]["filters"]["TripDomain"].as_array().unwrap().iter().filter_map(|v| v.as_str()).map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(","))
        };


    // -----------------------------------------------------------------------
    // Fourth: Page specific data
    // -----------------------------------------------------------------------

        match page {
            "explore" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/explore/title").and_then(|v| v.as_str()).unwrap_or("Explore"),
                    "template": TEMPLATE_EXPLORE,
                    "queries": [
                        ["explore", QUERY_EXPLORE.to_string().replace("/*","").replace("*/","")
                        .replace_filter("(TripDomain)", &render_structure["all"]["query_params"]["f"])
                        .replace_filter("(ParticipantGroup)", &render_structure["all"]["query_params"]["f"])],
                    ]});
            }
            "overview:year" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/overview/year").and_then(|v| v.as_str()).unwrap_or("Overview: Year"),
                    "template": TEMPLATE_OVERVIEW_YEAR,
                    "queries": [
                        ["overviewYear", QUERY_OVERVIEW_YEAR.to_string().replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)]
                    ]});
            }
            "overview:country" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/overview/country").and_then(|v| v.as_str()).unwrap_or("Overview: Country"),
                    "template": TEMPLATE_OVERVIEW_COUNTRY,
                    "queries": [
                         // Replace "c.Continent = 'Europa'" in QUERY_OVERVIEW_COUNTRY with value from settings in future version
                         ["overviewCountry", QUERY_OVERVIEW_COUNTRY.to_string().replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)]
                     ]});
            }
            "overview:plain" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/overview/plain").and_then(|v| v.as_str()).unwrap_or("Overview: Plain"),
                    "template": TEMPLATE_OVERVIEW_PLAIN,
                    "queries": [
                        ["overviewYear", QUERY_OVERVIEW_YEAR.to_string().replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)],
                         // Replace "c.Continent = 'Europa'" in QUERY_OVERVIEW_COUNTRY with value from settings in future version
                        ["overviewCountry", QUERY_OVERVIEW_COUNTRY.to_string().replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)]
                    ]});
            }
            "map" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/map/title").and_then(|v| v.as_str()).unwrap_or("Map"),
                    "template": TEMPLATE_MAP,
                    "queries": [
                        ["map_country_list", QUERY_MAP_COUNTRY_LIST.to_string()],
                        ["map_theme", QUERY_MAP_THEME.to_string()],
                        ["map_contour", QUERY_MAP_CONTOUR.to_string()],
                        ["map_country", QUERY_MAP_COUNTRY.to_string()],
                        ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS.to_string()],
                    ]});
                // See later in code for special cases
            }
            "statistics:summary" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/statistics/summary").and_then(|v| v.as_str()).unwrap_or("Statistics: Summary"),
                    "template": TEMPLATE_STATISTICS_SUMMARY,
                    "queries": [
                        ["statistics_visits", QUERY_STATISTICS_VISITS.replace("SELECT\n    Country,\n    OL,\n    SS,\n    VSS,\n    PS,\n    OLMQ,\n    SSMQ,\n    VSSMQ,\n    PSMQ\nFROM Aggregated\nORDER BY OL DESC;", "SELECT COUNT(DISTINCT Country) AS TripCount FROM Aggregated;").replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)],
                        ["statistics_trip_count", QUERY_STATISTICS_TRIP_COUNT.replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)],
                        ["statistics_per_domain_year", QUERY_STATISTICS_PER_DOMAIN_YEAR.replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)],
                        ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS.to_string()],
                    ]});
            }
            "statistics:visits" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/statistics/visits").and_then(|v| v.as_str()).unwrap_or("Statistics: Visits"),
                    "template": TEMPLATE_STATISTICS_VISITS,
                    "queries": [
                        ["statistics_visits", QUERY_STATISTICS_VISITS.replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)]
                    ]});
            }
            "statistics:overnights" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/statistics/overnights").and_then(|v| v.as_str()).unwrap_or("Statistics: Overnights"),
                    "template": TEMPLATE_STATISTICS_OVERNIGHTS,
                    "queries": [
                        ["statistics_overnights", QUERY_STATISTICS_OVERNIGHTS.replace("/*","").replace("*/","")
                        .replace("(ParticipantGroup)", &participant_group)
                        .replace("(TripDomain)", &trip_domain)],
                    ]});
            }
            "statistics:themes" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/themes/title").and_then(|v| v.as_str()).unwrap_or("Themes"),
                    "template": TEMPLATE_STATISTICS_THEMES,
                    "queries": [
                         ["statistics_theme_count", QUERY_STATISTICS_THEME_COUNT],
                    ]});
            }
            "dataset" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/dataset/title").and_then(|v| v.as_str()).unwrap_or("Dataset"),
                    "settings": render_structure["all"]["settings"],
                    "template": TEMPLATE_DATASET,
                    });
            }
            "more:source" => {
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/source/title").and_then(|v| v.as_str()).unwrap_or("Source"),
                    "template": TEMPLATE_SOURCE,
                    });
                render_structure["all"]["db_loaded"] = json!(if !&db_bytes.is_empty() { "stored" } else { "missing" });
            }
            "more:about" => {
                // Lägg till versionskontroll
                render_structure["page"] = json!({
                    "title": render_structure.pointer("/all/translation/about/title").and_then(|v| v.as_str()).unwrap_or("About"),
                    "template": TEMPLATE_ABOUT,
                    });
                render_structure["all"]["current_version"] = filecontent::fetch_text("version").await.into();
                render_structure["all"]["latest_version"] = json!(helper::get_latest_version_number().await);
            }
            _ => {
            
                web_sys::console::log_1(&"Second tier.".into());
                
                if let Some(suffix) = page.strip_prefix("trip:") {
                    // Title med outer id + dagbok + pass
                    render_structure["page"] = json!({
                        "title": suffix,
                        "template": TEMPLATE_TRIP,
                        "queries": [
                            ["trip_summary", QUERY_TRIP_SUMMARY.to_string().replace("/*_OUTER_ID_*/",suffix)],
                            ["trip_events", QUERY_TRIP_EVENTS.to_string().replace("/*_OUTER_ID_*/",suffix)],
                            ["trip_all_trips", QUERY_TRIP_ALL_TRIPS.to_string()],
                            ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS.to_string()],
                            // Lägg till filter
                            ["trip_border_crossings", QUERY_TRIP_BORDER_CROSSINGS.replace("/*","").replace("*/","").replace("_OUTER_ID_",suffix)],
                            ["trip_map_pins", QUERY_TRIP_MAP_PINS.replace("/*","").replace("*/","").replace("_OUTER_ID_",suffix)],
                            ["trip_previous", QUERY_TRIP_PREVIOUS.replace("/*_OUTER_ID_*/",suffix)],
                            ["trip_next", QUERY_TRIP_NEXT.replace("/*_OUTER_ID_*/",suffix)],
                    ]});
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
                    
                    }
                    // Title med outer id + dagbok + pass
                    render_structure["page"] = json!({
                        "title": render_structure.pointer("/all/translation/map/title").and_then(|v| v.as_str()).unwrap_or("Map"),
                        "template": TEMPLATE_MAP,
                        "queries": [
                            ["map_country_list", QUERY_MAP_COUNTRY_LIST.to_string()],
                            ["map_contour", QUERY_MAP_CONTOUR.to_string()],
                            ["map_country", QUERY_MAP_COUNTRY.to_string()],
                            ["map_theme", QUERY_MAP_THEME.to_string()],
                            ["common_trip_domains", QUERY_COMMON_TRIP_DOMAINS.to_string()],
                        ]});
                }
                
                if let Some(suffix) = page.strip_prefix("search:") {
                    // Title med outer id + dagbok + pass
                    render_structure["page"] = json!({
                        "title": suffix,
                        "template": TEMPLATE_SEARCH,
                        "settings": serde_json::to_value(&settings_response["settings"]).unwrap(),
                        "queries": [
                            ["search_trip", QUERY_SEARCH_TRIP.to_string().replace("/*_STRING_*/", suffix)],
                            ["search_event", QUERY_SEARCH_EVENT.to_string().replace("/*_STRING_*/", suffix)],
                        ]});
                }
                
            }
        }

    // -----------------------------------------------------------------------
    // Fourth: Render content
    // -----------------------------------------------------------------------

        web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]).unwrap().into());
        prepare_rendering(db_bytes, render_structure).await;

    });

}

pub async fn prepare_rendering(db_bytes: Vec<u8>, render_structure: serde_json::Value) {


    // SET TITLE  -----------------------------------------------------------------------

    //web_sys::console::log_1(&"----------------------".into());
    let title = render_structure["page"]["title"].as_str().unwrap_or("Default Title");
    web_sys::window().unwrap().document().unwrap().set_title(&format!("{title} - Immer in Bewegung"));
    //web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["title"]).unwrap().into());

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
    //web_sys::console::log_1(&serde_json::to_string(&query_response).unwrap().into());

    // Start with a clone of the "all" section from render_structure
    let mut merged_structure = render_structure["all"].clone();

    // Merge if both are objects
    match (&mut merged_structure, query_response) {
        (serde_json::Value::Object(ref mut target), serde_json::Value::Object(source)) => {
            for (k, v) in source {
                target.insert(k, v);
            }
        }
        // If either is not an object, fallback to keeping query_response as is
        (_, other) => {
            merged_structure = other;
        }
    }

    // RENDER TO 'APP'  -----------------------------------------------------------------------
    let _ = render::render2dom(&render_structure["page"]["template"].as_str().expect("template must be a string"), &merged_structure, "app");
    //web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["latest_version"]).unwrap().into());
    //helper::apply_preselected(&render_structure["all"]["query_params"]["f"]);
    //helper::attach_select_listeners();
    
    helper::apply_filter_from_opfs_to_selects();
    helper::attach_select_listener();

}