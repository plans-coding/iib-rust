use wasm_bindgen::prelude::*;
use serde_json::json;
use helper::SqlFilterReplace;

mod filecontent;
mod sqlite_query;
mod render;
mod query_params;
mod helper;

// Templates
const TEMPLATE_EXPLORE: &str = include_str!("../templates/explore.tera");
const TEMPLATE_OVERVIEW_YEAR: &str = include_str!("../templates/overview_year.tera");
const TEMPLATE_OVERVIEW_COUNTRY: &str = include_str!("../templates/overview_country.tera");
const TEMPLATE_OVERVIEW_PLAIN: &str = include_str!("../templates/overview_plain.tera");
const TEMPLATE_TRIP: &str = include_str!("../templates/trip.tera");
const TEMPLATE_IMAGES: &str = include_str!("../templates/images.tera");
const TEMPLATE_MAP: &str = include_str!("../templates/map.tera");
const TEMPLATE_SEARCH: &str = include_str!("../templates/search.tera");
const TEMPLATE_STATISTICS: &str = include_str!("../templates/statistics.tera");
const TEMPLATE_DATASET: &str = include_str!("../templates/dataset.tera");
const TEMPLATE_ABOUT: &str = include_str!("../templates/about.tera");
const TEMPLATE_CONFIGURE: &str = include_str!("../templates/_configure.tera");

// Advanced queries
const QUERY_EXPLORE: &str = include_str!("../queries/explore.sql");
const QUERY_OVERVIEW_YEAR: &str = include_str!("../queries/overview_year.sql");
const QUERY_OVERVIEW_COUNTRY: &str = include_str!("../queries/overview_country.sql");
const QUERY_STATISTICS_VISITS: &str = include_str!("../queries/statistics_visits.sql");
const QUERY_STATISTICS_BORDER_CROSSINGS: &str = include_str!("../queries/statistics_border_crossings.sql");
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
const QUERY_TRIP_BORDER_CROSSINGS: &str = include_str!("../queries/simple/trip_border_crossings.sql");
const QUERY_TRIP_EVENTS: &str = include_str!("../queries/simple/trip_events.sql");
const QUERY_TRIP_SUMMARY: &str = include_str!("../queries/simple/trip_summary.sql");


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
            // Set query parameter 'p' to 'configure'
            query_params::set_query_params(&json!({"path":"configure"}));
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

        // If "p" is missing or empty, set it to "overview"
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
        let translation_content = filecontent::fetch_json(&translation_filename_extracted).await;
        //web_sys::console::log_1(&serde_json::to_string(&translation_content).unwrap().into());

        // Get all settings
        let settings_query = vec![
            ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string())
        ];
        let settings_response = sqlite_query::get_query_data(&db_bytes, settings_query).await;


        render_structure["all"]["settings"] = serde_json::to_value(&settings_response["settings"]).unwrap();
        render_structure["all"]["translation"] = translation_content.expect("Error with translation data.");
        //web_sys::console::log_1(&serde_json::to_string(&render_structure).unwrap().into());

    // -----------------------------------------------------------------------
    // Fourth: Page specific data
    // -----------------------------------------------------------------------

        match page {
            "explore" => {
                render_structure["page"] = json!({
                    "title": "Explore",
                    "app": {
                        "template": TEMPLATE_EXPLORE,
                        "queries": {
                            "chronik.db": [
                                ["explore", QUERY_EXPLORE.to_string().replace("/*","").replace("*/","")
                                .replace_filter("(TripDomain)", &render_structure["all"]["query_params"]["f"])
                                .replace_filter("(ParticipantGroup)", &render_structure["all"]["query_params"]["f"])],
                                ["tripDomains", QUERY_COMMON_TRIP_DOMAINS.to_string(), ""],
                                ["participantGroups", QUERY_COMMON_PARTICIPANT_GROUPS.to_string(), ""],
                            ]}}});
            }
            "overview:year" => {
                render_structure["page"] = json!({
                    "title": "Overview: Year",
                    "app": {
                        "template": TEMPLATE_OVERVIEW_YEAR,
                        "queries": {
                            "chronik.db": [
                                ["overviewYear", QUERY_OVERVIEW_YEAR.to_string().replace("/*","").replace("*/","")
                                .replace_filter("(TripDomain)", &render_structure["all"]["query_params"]["f"])
                                .replace_filter("(ParticipantGroup)", &render_structure["all"]["query_params"]["f"])],
                                ["tripDomains", QUERY_COMMON_TRIP_DOMAINS.to_string(), ""],
                                ["participantGroups", QUERY_COMMON_PARTICIPANT_GROUPS.to_string(), ""],
                    ]}}});
            }
            "overview:country" => {
                render_structure["page"] = json!({
                    "title": "Overview: Country",
                    "app": {
                        "template": TEMPLATE_OVERVIEW_COUNTRY,
                        "queries": {
                            "chronik.db": [
                                 // Replace "c.Continent = 'Europa'" in QUERY_OVERVIEW_COUNTRY with value from settings in future version
                                 ["overviewCountry", QUERY_OVERVIEW_COUNTRY.to_string().replace("/*","").replace("*/","")
                                 .replace_filter("(TripDomain)", &render_structure["all"]["query_params"]["f"])
                                 .replace_filter("(ParticipantGroup)", &render_structure["all"]["query_params"]["f"])],
                                 ["tripDomains", QUERY_COMMON_TRIP_DOMAINS.to_string(), ""],
                                 ["participantGroups", QUERY_COMMON_PARTICIPANT_GROUPS.to_string(), ""],
                            ]}}});
            }
            "overview:plain" => {
                render_structure["page"] = json!({
                    "title": "Overview: Plain",
                    "app": {
                        "template": TEMPLATE_OVERVIEW_PLAIN,
                        "queries": {
                            "chronik.db": [
                                ["overviewYear", QUERY_OVERVIEW_YEAR.to_string().replace("/*","").replace("*/","")
                                .replace_filter("(TripDomain)", &render_structure["all"]["query_params"]["f"])
                                .replace_filter("(ParticipantGroup)", &render_structure["all"]["query_params"]["f"])],
                                 // Replace "c.Continent = 'Europa'" in QUERY_OVERVIEW_COUNTRY with value from settings in future version
                                 ["overviewCountry", QUERY_OVERVIEW_COUNTRY.to_string().replace("/*","").replace("*/","")
                                 .replace_filter("(TripDomain)", &render_structure["all"]["query_params"]["f"])
                                 .replace_filter("(ParticipantGroup)", &render_structure["all"]["query_params"]["f"])],
                                 ["tripDomains", QUERY_COMMON_TRIP_DOMAINS.to_string(), ""],
                                 ["participantGroups", QUERY_COMMON_PARTICIPANT_GROUPS.to_string(), ""],
                            ]}}});
            }
            "trip" => {
                // Title med outer id + dagbok + pass
                render_structure["page"] = json!({
                    "title": "?",
                    "app": {
                        "template": TEMPLATE_TRIP,
                        "queries": {
                            "chronik.db": [
                                ["trip_summary", QUERY_TRIP_SUMMARY.to_string(), ""],
                                ["trip_events", QUERY_TRIP_EVENTS.to_string(), ""],
                                ["trip_allTrips", QUERY_TRIP_ALL_TRIPS.to_string(), ""],
                                ["tripDomains", QUERY_COMMON_TRIP_DOMAINS.to_string(), ""],
                                // Lägg till filter
                                ["trip_borderCrossings", QUERY_STATISTICS_BORDER_CROSSINGS, ""],
                                ["trip_mapPins", QUERY_TRIP_MAP_PINS, ""],
                ]}}});
            }
            "images" => {
                render_structure["page"] = json!({
                    "title": "Overview",
                    "app": {
                        "template": TEMPLATE_IMAGES,
                        "queries": {
                            "chronik.db": [
                                ["images_dateList", QUERY_IMAGES_DATE_LIST.to_string(), ""],
                                ["common_tripDomains", QUERY_COMMON_TRIP_DOMAINS.to_string(), ""],
                                ["images_photoTime", QUERY_IMAGES_PHOTO_TIME.to_string(), ""],
                                ["tripDomains", QUERY_COMMON_TRIP_DOMAINS.to_string(), ""],
                ]}}});
            }
            "map" => {
                render_structure["page"] = json!({
                    "title": "Overview",
                    "app": {
                        "template": TEMPLATE_MAP,
                        "queries": {
                            "chronik.db": [
                                ["map_countryList", QUERY_MAP_COUNTRY_LIST.to_string(), ""],
                                ["map_theme", QUERY_MAP_THEME.to_string(), ""],
                                ["map_contour", QUERY_MAP_CONTOUR.to_string(), ""],
                                ["map_country", QUERY_MAP_COUNTRY.to_string(), ""],
                                ["tripDomains", QUERY_COMMON_TRIP_DOMAINS.to_string(), ""],
                    ]}}});
            }
            "statistics" => {
                render_structure["page"] = json!({
                    "title": "Overview",
                    "app": {
                        "template": TEMPLATE_STATISTICS,
                        "queries": {
                            "chronik.db": [
                                ["statistics_perDomainYear", QUERY_STATISTICS_PER_DOMAIN_YEAR, ""],
                                ["statistics_tripCount", QUERY_STATISTICS_TRIP_COUNT.to_string(), ""],
                                ["statistics_overnights", QUERY_STATISTICS_OVERNIGHTS, ""],
                                ["common_tripDomains", QUERY_COMMON_TRIP_DOMAINS.to_string(), ""],
                                ["statistics_OLSSVSS", QUERY_STATISTICS_VISITS, ""],
                                ["statistics_theme_count", QUERY_STATISTICS_THEME_COUNT, ""],
                    ]}}});
            }
            "dataset" => {
                render_structure["page"] = json!({
                    "title": "Overview",
                    "app": {
                        "template": TEMPLATE_DATASET,
                    }});
            }
            "configure" => {
                render_structure["page"] = json!({
                    "title": "Overview",
                    "app": {
                        "template": TEMPLATE_CONFIGURE,
                    }});
            }
            "about" => {
                // Lägg till versionskontroll
                render_structure["page"] = json!({
                    "title": "Overview",
                    "app": {
                        "template": TEMPLATE_ABOUT,
                    },
                });
                render_structure["all"]["current_version"] = filecontent::fetch_text("version").await.into();
                render_structure["all"]["latest_version"] = json!(helper::get_latest_version_number().await);
            }
            "search" => {
                render_structure["page"] = json!({
                    "title": "translation.menu queryParams p",
                    "app": {
                        "template": TEMPLATE_SEARCH,
                        "queries": {
                            "chronik.db": [
                                ["search_trip", QUERY_SEARCH_TRIP.to_string(), ""],
                                ["search_event", QUERY_SEARCH_EVENT.to_string(), ""],
                                ["common_tripDomains", QUERY_COMMON_TRIP_DOMAINS.to_string(), ""],
                    ]}}});
            }
            _ => {
                web_sys::console::log_1(&"Another error.".into());
            }
        }

    // -----------------------------------------------------------------------
    // Fourth: Render content
    // -----------------------------------------------------------------------

        prepare_rendering(db_bytes, render_structure).await;

    });

}

pub async fn prepare_rendering(db_bytes: Vec<u8>, render_structure: serde_json::Value) {


    // SET TITLE  -----------------------------------------------------------------------

    //web_sys::console::log_1(&"----------------------".into());
    let title = render_structure["page"]["title"].as_str().unwrap_or("Default Title");
    web_sys::window().unwrap().document().unwrap().set_title(title);
    web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["title"]).unwrap().into());


    // RENDER TO 'MENU'  -----------------------------------------------------------------------

    //web_sys::console::log_1(&"----------------------".into());
    //web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["menu"]).unwrap().into());
    //render::render2dom(TEMPLATE_MENU, &render_structure["all"], "menu");

    // RUN SQLITE QUERIES  -----------------------------------------------------------------------

    //web_sys::console::log_1(&"----------------------".into());
    //web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["app"]["queries"]["chronik.db"]).unwrap().into());

    let combined_query: Vec<(String, String)> = render_structure["page"]["app"]["queries"]["chronik.db"]
    .as_array().unwrap_or(&Vec::new()).iter().map(|row| {
        // Each row: [key, value]
        let k = row[0].as_str().unwrap_or("").to_string();
        let v = row[1].as_str().unwrap_or("").to_string();
        (k, v)
    })
    .collect();

    let query_response: serde_json::Value = sqlite_query::get_query_data(&db_bytes, combined_query).await;
    web_sys::console::log_1(&serde_json::to_string(&query_response).unwrap().into());

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

    //web_sys::console::log_1(&"----------------------".into());
    render::render2dom(&render_structure["page"]["app"]["template"].as_str().expect("template must be a string"), &merged_structure, "app");
    //web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["latest_version"]).unwrap().into());
    helper::apply_preselected(&render_structure["all"]["query_params"]["f"]);
    helper::attach_select_listeners();



}


