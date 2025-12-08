// Web assembly
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

//use web_sys::{Response, window};
use serde_json::json;
use liquid::model::Value; // Needed for translation
use liquid::model::Object; // Needed for translation
use liquid::ValueView;

mod filecontent;
mod query;
mod render;
mod pagelogic;

// Templates
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

// Queries
// -- Overview
const QUERY_OVERVIEW_YEAR: &str = include_str!("../queries/overview_year.sql");
const QUERY_OVERVIEW_COUNTRY: &str = include_str!("../queries/overview_country.sql");
// -- Statistics
const QUERY_STATISTICS_VISITS: &str = include_str!("../queries/statistics_visits.sql");
const QUERY_STATISTICS_BORDER_CROSSINGS: &str = include_str!("../queries/statistics_border_crossings.sql");
const QUERY_STATISTICS_OVERNIGHTS: &str = include_str!("../queries/statistics_overnights.sql");
const QUERY_STATISTICS_PER_DOMAIN_YEAR: &str = include_str!("../queries/statistics_per_domain_year.sql");
const QUERY_STATISTICS_THEME_COUNT: &str = include_str!("../queries/statistics_theme_count.sql");
// -- Map
const QUERY_TRIP_MAP_PINS: &str = include_str!("../queries/trip_map_pins.sql");

#[wasm_bindgen(start)]
fn start() {

    wasm_bindgen_futures::spawn_local(async {

        let queryParams = pagelogic::get_all_query_params();
        web_sys::console::log_1(&format!("query params = {:?}", queryParams.get("p").map(|s| s.as_str()).unwrap_or("")).into());

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

        use serde_json::Value as JsonValue;

        // Run the query
        let translation_query = vec![
            ("settings".to_string(), "SELECT Value FROM bewxx_Settings WHERE AttributeGroup = 'Base' AND Attribute = 'LanguageFile';".to_string())
        ];
        let translation_filename = query::get_query_data(&db_bytes, translation_query).await;

        let json_obj: JsonValue = serde_json::to_value(&translation_filename).unwrap();

        let translation_filename_extracted = format!(
            "languages/{}",
            json_obj["settings"][0]["Value"]
            .as_str()
            .expect("Expected settings[0].Value to be a string")
        );

        web_sys::console::log_1(&translation_filename_extracted.as_str().into());

        let translation_content = filecontent::fetch_json(&translation_filename_extracted).await;

        web_sys::console::log_1(
            &serde_json::to_string(&translation_content).unwrap().into()
        );

        let mut liquid_obj = liquid::Object::new();

        match queryParams.get("p").map(|s| s.as_str()).unwrap_or("") {
            "" | "overview" => {

                let queries = vec![
                    ("overviewYear".to_string(), QUERY_OVERVIEW_YEAR.to_string()),
                    ("overviewCountry".to_string(), QUERY_OVERVIEW_COUNTRY.to_string()),
                    ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
                    ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
                    ("participantGroups".to_string(), "SELECT * FROM bewx_ParticipantGroups;".to_string())
                ];
                liquid_obj = query::get_query_data(&db_bytes, queries).await;

                liquid_obj.insert(
                    "translation".into(),
                                  translation_content
                                  .as_ref()
                                  .map(|content| liquid::model::Value::Object(liquid::model::to_object(content).unwrap()))
                                  .unwrap_or(liquid::model::Value::Nil),
                );

                render::render_to_dom(TEMPLATE_OVERVIEW, &liquid_obj, "app");


                // TIME
                use kstring::KString;
                use chrono::Local;


                let now = Local::now();
                let time: Object = [
                    ("now_year", now.format("%Y").to_string()),
                                      ("now_date", now.format("%Y-%m-%d").to_string())
                ].into_iter()
                .map(|(k, v)| (KString::from(k), Value::scalar(v)))
                .collect();

                liquid_obj.insert("time".into(), Value::Object(time));


                let mut obj = Object::new();
                for (k, v) in queryParams {
                    obj.insert(k.into(), Value::scalar(v));
                }
                liquid_obj.insert("queryParams".into(), Value::Object(obj));




                web_sys::console::log_1(
                    &serde_json::to_string(&liquid_obj).unwrap().into()
                );

                render::render_to_dom(TEMPLATE_MENU, &liquid_obj, "menu");

            }
            "images" => {

                // TIME
                use kstring::KString;
                use chrono::Local;


                let now = Local::now();
                let time: Object = [
                    ("now_year", now.format("%Y").to_string()),
                                      ("now_date", now.format("%Y-%m-%d").to_string())
                ].into_iter()
                .map(|(k, v)| (KString::from(k), Value::scalar(v)))
                .collect();

                liquid_obj.insert("time".into(), Value::Object(time));


                let mut obj = Object::new();
                for (k, v) in queryParams {
                    obj.insert(k.into(), Value::scalar(v));
                }
                liquid_obj.insert("queryParams".into(), Value::Object(obj));


                //let queries: Vec<(String, String)> = vec![];
                let queries = vec![
                    ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
                                      ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
                ];
                liquid_obj = query::get_query_data(&db_bytes, queries).await;

                liquid_obj.insert(
                    "translation".into(),
                                  translation_content
                                  .as_ref()
                                  .map(|content| liquid::model::Value::Object(liquid::model::to_object(content).unwrap()))
                                  .unwrap_or(liquid::model::Value::Nil),
                );

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

                liquid_obj.insert(
                    "translation".into(),
                                  translation_content
                                  .as_ref()
                                  .map(|content| liquid::model::Value::Object(liquid::model::to_object(content).unwrap()))
                                  .unwrap_or(liquid::model::Value::Nil),
                );

                render::render_to_dom(TEMPLATE_STATISTICS, &liquid_obj, "app");


                // TIME
                use kstring::KString;
                use chrono::Local;


                let now = Local::now();
                let time: Object = [
                    ("now_year", now.format("%Y").to_string()),
                                      ("now_date", now.format("%Y-%m-%d").to_string())
                ].into_iter()
                .map(|(k, v)| (KString::from(k), Value::scalar(v)))
                .collect();

                liquid_obj.insert("time".into(), Value::Object(time));


                let mut obj = Object::new();
                for (k, v) in queryParams {
                    obj.insert(k.into(), Value::scalar(v));
                }
                liquid_obj.insert("queryParams".into(), Value::Object(obj));




                web_sys::console::log_1(
                    &serde_json::to_string(&liquid_obj).unwrap().into()
                );

                render::render_to_dom(TEMPLATE_MENU, &liquid_obj, "menu");

            }
            "about" => {
                //let queries: Vec<(String, String)> = vec![];
                let queries = vec![
                                      ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
                                      ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
                ];
                liquid_obj = query::get_query_data(&db_bytes, queries).await;

                liquid_obj.insert(
                    "translation".into(),
                                  translation_content
                                  .as_ref()
                                  .map(|content| liquid::model::Value::Object(liquid::model::to_object(content).unwrap()))
                                  .unwrap_or(liquid::model::Value::Nil),
                );

                render::render_to_dom(TEMPLATE_ABOUT, &liquid_obj, "app");


                // TIME
                use kstring::KString;
                use chrono::Local;


                let now = Local::now();
                let time: Object = [
                    ("now_year", now.format("%Y").to_string()),
                                      ("now_date", now.format("%Y-%m-%d").to_string())
                ].into_iter()
                .map(|(k, v)| (KString::from(k), Value::scalar(v)))
                .collect();

                liquid_obj.insert("time".into(), Value::Object(time));


                let mut obj = Object::new();
                for (k, v) in queryParams {
                    obj.insert(k.into(), Value::scalar(v));
                }
                liquid_obj.insert("queryParams".into(), Value::Object(obj));




                web_sys::console::log_1(
                    &serde_json::to_string(&liquid_obj).unwrap().into()
                );

                render::render_to_dom(TEMPLATE_MENU, &liquid_obj, "menu");


            }
            "configure" => {
                //let queries: Vec<(String, String)> = vec![];
                let queries = vec![
                    ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
                    ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
                ];
                liquid_obj = query::get_query_data(&db_bytes, queries).await;

                liquid_obj.insert(
                    "translation".into(),
                                  translation_content
                                  .as_ref()
                                  .map(|content| liquid::model::Value::Object(liquid::model::to_object(content).unwrap()))
                                  .unwrap_or(liquid::model::Value::Nil),
                );

                render::render_to_dom(TEMPLATE_CONFIGURE, &liquid_obj, "app");


                // TIME
                use kstring::KString;
                use chrono::Local;


                let now = Local::now();
                let time: Object = [
                    ("now_year", now.format("%Y").to_string()),
                                      ("now_date", now.format("%Y-%m-%d").to_string())
                ].into_iter()
                .map(|(k, v)| (KString::from(k), Value::scalar(v)))
                .collect();

                liquid_obj.insert("time".into(), Value::Object(time));


                let mut obj = Object::new();
                for (k, v) in queryParams {
                    obj.insert(k.into(), Value::scalar(v));
                }
                liquid_obj.insert("queryParams".into(), Value::Object(obj));




                web_sys::console::log_1(
                    &serde_json::to_string(&liquid_obj).unwrap().into()
                );

                render::render_to_dom(TEMPLATE_MENU, &liquid_obj, "menu");


            }
            _ => {
                //content.set_inner_html("<p>Page not found</p>");
            }
        }









        // Lägg till upload sida + funktion i Rust


    });

}
