use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use serde_json::{json, Value};
use kstring::KString;

mod filecontent;
mod sqlite_query;
mod render;
mod query_params;
mod helper;

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

const QUERY_OVERVIEW_YEAR: &str = include_str!("../queries/overview_year.sql");
const QUERY_OVERVIEW_COUNTRY: &str = include_str!("../queries/overview_country.sql");

const QUERY_STATISTICS_VISITS: &str = include_str!("../queries/statistics_visits.sql");
const QUERY_STATISTICS_BORDER_CROSSINGS: &str = include_str!("../queries/statistics_border_crossings.sql");
const QUERY_STATISTICS_OVERNIGHTS: &str = include_str!("../queries/statistics_overnights.sql");
const QUERY_STATISTICS_PER_DOMAIN_YEAR: &str = include_str!("../queries/statistics_per_domain_year.sql");
const QUERY_STATISTICS_THEME_COUNT: &str = include_str!("../queries/statistics_theme_count.sql");

const QUERY_TRIP_MAP_PINS: &str = include_str!("../queries/trip_map_pins.sql");


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
            query_params::set_query_params(&json!({"p":"configure"}));
            // set page = configure -- needed?
        }

    // -----------------------------------------------------------------------
    // Second: Handle query parameters
    // -----------------------------------------------------------------------

        let mut query_params = query_params::get_query_params();
        // Presume ?p=overview if not set at all
        let page = match &query_params["p"] { serde_json::Value::String(s) if !s.is_empty() => s.as_str(), _ => "overview", };
        web_sys::console::log_1(&format!("Loading page: {}",page).into());

    // -----------------------------------------------------------------------
    // Third: Common data for all pages
    // -----------------------------------------------------------------------

        let mut render_structure = json!({});
        render_structure["all"]["query_params"] = query_params.clone();
        render_structure["all"]["time"] = helper::build_time_json();

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
        web_sys::console::log_1(&serde_json::to_string(&render_structure).unwrap().into());

    // -----------------------------------------------------------------------
    // Fourth: Page specific data
    // -----------------------------------------------------------------------

        match page {
            "overview" => {
                render_structure["page"] = json!({
                    "title": "Overview",
                    "menu": TEMPLATE_MENU,
                    "app": {
                        "template": TEMPLATE_OVERVIEW,
                        "queries": {
                            "chronik.db": [
                                //["settings", "SELECT * FROM bewxx_Settings;".to_string(), ""],
                                ["overviewYear", QUERY_OVERVIEW_YEAR.to_string(), ""],
                                ["overviewCountry", QUERY_OVERVIEW_COUNTRY.to_string(), ""],
                                ["tripDomains", "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string(), ""],
                                ["participantGroups", "SELECT * FROM bewx_ParticipantGroups;".to_string(), ""],
            ]}}});}
            _ => {
                //content.set_inner_html("<p>Page not found</p>");
            }
        }

    // -----------------------------------------------------------------------
    // Fourth: Render content
    // -----------------------------------------------------------------------

        //render::
        prepare_rendering(db_bytes, render_structure);

    });

}

pub fn prepare_rendering(db_bytes: Vec<u8>, render_structure: serde_json::Value) {


    // SET TITLE  -----------------------------------------------------------------------

    web_sys::console::log_1(&"----------------------".into());
    let title = render_structure["page"]["title"].as_str().unwrap_or("Default Title");
    web_sys::window().unwrap().document().unwrap().set_title(title);
    web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["title"]).unwrap().into());


    // RENDER TO 'MENU'  -----------------------------------------------------------------------

    web_sys::console::log_1(&"----------------------".into());
    web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["menu"]).unwrap().into());

    let menu_liquid: liquid::model::Object = json_to_liquid_object(&render_structure["all"]);
    render::render2dom(TEMPLATE_MENU, &menu_liquid, "menu");


    // RUN SQLITE QUERIES  -----------------------------------------------------------------------

    web_sys::console::log_1(&"----------------------".into());
    web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["app"]["queries"]["chronik.db"]).unwrap().into());

    /*let settings_query = vec![
        ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string())
    ];
    let settings_response = sqlite_query::get_query_data(&db_bytes, settings_query).await;
    */


    // RENDER TO 'APP'  -----------------------------------------------------------------------

    //web_sys::console::log_1(&"----------------------".into());
    //web_sys::console::log_1(&serde_json::to_string(&render_structure["page"]["app"]["template"]).unwrap().into());

}

fn json_to_liquid_object(v: &serde_json::Value) -> liquid::model::Object {
    match v {
        serde_json::Value::Object(obj) => obj.iter()
        .map(|(k, v)| (KString::from(k.clone()), json_value_to_liquid(v)))
        .collect(),
        _ => liquid::model::Object::new(), // if not an object, return empty
    }
}

fn json_value_to_liquid(v: &serde_json::Value) -> liquid::model::Value {
    match v {
        serde_json::Value::Null => liquid::model::Value::Nil,
        serde_json::Value::Bool(b) => liquid::model::Value::Scalar((*b).into()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                liquid::model::Value::Scalar(i.into())
            } else if let Some(f) = n.as_f64() {
                liquid::model::Value::Scalar(f.into())
            } else {
                liquid::model::Value::Nil
            }
        }
        serde_json::Value::String(s) => liquid::model::Value::Scalar(s.clone().into()),
        serde_json::Value::Array(arr) => liquid::model::Value::Array(
            arr.iter().map(json_value_to_liquid).collect()
        ),
        serde_json::Value::Object(obj) => {
            let map: liquid::model::Object = obj.iter()
            .map(|(k, v)| (KString::from(k.clone()), json_value_to_liquid(v)))
            .collect();
            liquid::model::Value::Object(map)
        }
    }
}




        //use serde_yaml::Value;

        /*

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
            "dataset" => {
                let queries = vec![
                    ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
                    ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
                ];
                liquid_obj = query::get_query_data(&db_bytes, queries).await;

                helper::insert_translation(&mut liquid_obj, &translation_content);
                render::render_to_dom(TEMPLATE_DATASET, &liquid_obj, "app");
                render::render_to_dom(TEMPLATE_MENU, &liquid_obj, "menu");
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

*/



//let queries: Vec<(String, String)> = vec![];
