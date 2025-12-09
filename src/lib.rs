use wasm_bindgen::prelude::*;
use std::collections::HashMap;

mod filecontent;
mod query;
mod render;
mod pagelogic;
mod helper;

// -- Template
const TEMPLATE_MENU: &str = include_str!("../templates/_menu.liquid");
const TEMPLATE_OVERVIEW: &str = include_str!("../templates/overview.liquid");
const TEMPLATE_TRIP: &str = include_str!("../templates/trip.liquid");
const TEMPLATE_IMAGES: &str = include_str!("../templates/images.liquid");
const TEMPLATE_MAP: &str = include_str!("../templates/map.liquid");
const TEMPLATE_SEARCH: &str = include_str!("../templates/search.liquid");
const TEMPLATE_STATISTICS: &str = include_str!("../templates/statistics.liquid");
const TEMPLATE_DATASET: &str = include_str!("../templates/dataset.liquid");
const TEMPLATE_SOURCE: &str = include_str!("../templates/source.liquid");
const TEMPLATE_ABOUT: &str = include_str!("../templates/about.liquid");
const TEMPLATE_CONFIGURE: &str = include_str!("../templates/_configure.liquid");
// -- Query Overview
const QUERY_OVERVIEW_YEAR: &str = include_str!("../queries/overview_year.sql");
const QUERY_OVERVIEW_COUNTRY: &str = include_str!("../queries/overview_country.sql");
// -- Query Statistics
const QUERY_STATISTICS_VISITS: &str = include_str!("../queries/statistics_visits.sql");
const QUERY_STATISTICS_BORDER_CROSSINGS: &str = include_str!("../queries/statistics_border_crossings.sql");
const QUERY_STATISTICS_OVERNIGHTS: &str = include_str!("../queries/statistics_overnights.sql");
const QUERY_STATISTICS_PER_DOMAIN_YEAR: &str = include_str!("../queries/statistics_per_domain_year.sql");
const QUERY_STATISTICS_THEME_COUNT: &str = include_str!("../queries/statistics_theme_count.sql");
// -- Query Map
const QUERY_TRIP_MAP_PINS: &str = include_str!("../queries/trip_map_pins.sql");

#[wasm_bindgen(start)]
fn start() {

    wasm_bindgen_futures::spawn_local(async {

        let mut query_params = pagelogic::get_all_query_params();

        // Presume ?p=overview if not set at all
        query_params
        .entry("p".to_string())
        .and_modify(|v| if v.is_empty() { *v = "overview".to_string() })
        .or_insert("overview".to_string());
        web_sys::console::log_1(&serde_json::to_string(&query_params).unwrap().into());

        // -----------------------------------------------------------------------
        // First: Get sqlite database binary
        // -----------------------------------------------------------------------

        let db_bytes = filecontent::get_sqlite_binary().await;
        // You can now use db_bytes, e.g. log length
        web_sys::console::log_1(&format!("DB size: {}", db_bytes.len()).into());

        // IF NOT db_bytes then go to ?p=configure

        // -----------------------------------------------------------------------
        // Second: Run queries in sqlite database
        // -----------------------------------------------------------------------

        // ---- Get correct translation for app
        let translation_query = vec![
            ("settings".to_string(), "SELECT Value FROM bewxx_Settings WHERE AttributeGroup = 'Base' AND Attribute = 'LanguageFile';".to_string())
        ];
        let translation_filename = query::get_query_data(&db_bytes, translation_query).await;
        let json_obj: serde_json::Value = serde_json::to_value(&translation_filename).unwrap();
        let translation_filename_extracted = format!("languages/{}",json_obj["settings"][0]["Value"].as_str().expect("Expected settings[0].Value to be a string"));
        web_sys::console::log_1(&translation_filename_extracted.as_str().into());
        let translation_content = filecontent::fetch_json(&translation_filename_extracted).await;
        web_sys::console::log_1(&serde_json::to_string(&translation_content).unwrap().into());

        let mut liquid_obj = liquid::Object::new();

        let page: &str = query_params.get("p").unwrap().as_str();

        // ---- Page specific code
        match page {
            "overview" => {
                let queries = vec![
                    ("overviewYear".to_string(), QUERY_OVERVIEW_YEAR.to_string()),
                    ("overviewCountry".to_string(), QUERY_OVERVIEW_COUNTRY.to_string()),
                    ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
                    ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
                    ("participantGroups".to_string(), "SELECT * FROM bewx_ParticipantGroups;".to_string())
                ];
                liquid_obj = query::get_query_data(&db_bytes, queries).await;

                helper::insert_translation(&mut liquid_obj, &translation_content);
                helper::insert_time(&mut liquid_obj);
                helper::insert_query_params(&mut liquid_obj, &query_params);
                render::render_to_dom(TEMPLATE_OVERVIEW, &liquid_obj, "app");

                render::render_to_dom(TEMPLATE_MENU, &liquid_obj, "menu");
                web_sys::console::log_1(&serde_json::to_string(&liquid_obj).unwrap().into());
            }
            "images" => {
                let queries = vec![
                    ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
                    ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
                ];
                liquid_obj = query::get_query_data(&db_bytes, queries).await;

                helper::insert_translation(&mut liquid_obj, &translation_content);
                render::render_to_dom(TEMPLATE_CONFIGURE, &liquid_obj, "app");
            }
            "map" => {

            }
            "statistics" => {
                let queries = vec![
                    ("statisticsVisits".to_string(), QUERY_STATISTICS_VISITS.to_string()),
                    ("statisticsBorderCrossings".to_string(), QUERY_STATISTICS_BORDER_CROSSINGS.to_string()),
                    ("statisticsOvernights".to_string(), QUERY_STATISTICS_OVERNIGHTS.to_string()),
                    ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
                    ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
                    ("participantGroups".to_string(), "SELECT * FROM bewx_ParticipantGroups;".to_string())
                ];
                liquid_obj = query::get_query_data(&db_bytes, queries).await;

                helper::insert_translation(&mut liquid_obj, &translation_content);
                render::render_to_dom(TEMPLATE_STATISTICS, &liquid_obj, "app");

                helper::insert_time(&mut liquid_obj);
                render::render_to_dom(TEMPLATE_MENU, &liquid_obj, "menu");
                web_sys::console::log_1(&serde_json::to_string(&liquid_obj).unwrap().into());
            }
            "about" => {
                let queries = vec![
                    ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
                    ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
                ];
                liquid_obj = query::get_query_data(&db_bytes, queries).await;

                helper::insert_translation(&mut liquid_obj, &translation_content);
                render::render_to_dom(TEMPLATE_ABOUT, &liquid_obj, "app");

                helper::insert_time(&mut liquid_obj);
                render::render_to_dom(TEMPLATE_MENU, &liquid_obj, "menu");
                web_sys::console::log_1(&serde_json::to_string(&liquid_obj).unwrap().into());
            }
            "configure" => {
                let queries = vec![
                    ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
                    ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
                ];
                liquid_obj = query::get_query_data(&db_bytes, queries).await;

                helper::insert_translation(&mut liquid_obj, &translation_content);
                render::render_to_dom(TEMPLATE_CONFIGURE, &liquid_obj, "app");

                helper::insert_time(&mut liquid_obj);
                render::render_to_dom(TEMPLATE_MENU, &liquid_obj, "menu");
                web_sys::console::log_1(&serde_json::to_string(&liquid_obj).unwrap().into());
            }
            _ => {
                //content.set_inner_html("<p>Page not found</p>");
            }
        }


        // Lägg till upload sida + funktion i Rust


    });

}


//let queries: Vec<(String, String)> = vec![];
