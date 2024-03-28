use serde_json::{Map as SerdeMap, Value};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};

#[tokio::main]
async fn main() {
    let base_url = String::from_str("https://gorzdrav.spb.ru/_api/api/v2").unwrap();
    let dir_name: PathBuf = ["mockData"].iter().collect();

    create_dir_all(dir_name.clone()).await.unwrap();

    let hospital_data = fetch_hospitals(&base_url, &dir_name).await;
    fetch_districts(&base_url, &dir_name).await;

    create_dir_all(dir_name.join("specialties")).await.unwrap();
    create_dir_all(dir_name.join("hospitals-specialties"))
        .await
        .unwrap();

    fetch_specialties(hospital_data, &base_url, &dir_name).await;
}

async fn fetch_hospitals(base_url: &String, dir_name: &Path) -> SerdeMap<String, Value> {
    let res = reqwest::get(format!("{}/shared/lpus", base_url))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let mut file = File::create(dir_name.join("hospitals.json")).await.unwrap();

    file.write_all(res.as_bytes()).await.unwrap();

    serde_json::from_str(&res).unwrap()
}

async fn fetch_districts(base_url: &String, dir_name: &Path) {
    let res = reqwest::get(format!("{}/shared/districts", base_url))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let mut file = File::create(dir_name.join("districts.json")).await.unwrap();

    file.write_all(res.as_bytes()).await.unwrap();
}

async fn fetch_specialties(
    hospital_data: SerdeMap<String, Value>,
    base_url: &String,
    dir_name: &Path,
) {
    let Some(hospitals) = hospital_data.get("result") else {
        return;
    };

    let Value::Array(hospitals) = hospitals else {
        return;
    };

    for hospital in hospitals {
        let Value::Object(hospital) = hospital else {
            continue;
        };

        let Some(hospital_id) = hospital.get("id") else {
            continue;
        };

        let Ok(hospital_id) = get_id_string(hospital_id) else {
            continue;
        };

        let specialties_data = fetch_specialties_for_hospital(&hospital_id, base_url, dir_name).await;

        let Some(specialties) = specialties_data.get("result") else {
            continue;
        };

        let Value::Array(specialties) = specialties else {
            continue;
        };

        for specialty in specialties {
            let Value::Object(specialty) = specialty else {
                continue;
            };

            let Some(specialty_id) = specialty.get("id") else {
                continue;
            };

            let Ok(specialty_id) = get_id_string(specialty_id) else {
                continue;
            };

            tokio::spawn(fetch_doctors_for_specialty(
                hospital_id.clone(),
                specialty_id,
                base_url.clone(),
                dir_name.to_path_buf(),
            ));
        }
    }
}

async fn fetch_specialties_for_hospital(
    hospital_id: &String,
    base_url: &String,
    dir_name: &Path,
) -> SerdeMap<String, Value> {
    let res = reqwest::get(format!(
        "{}/schedule/lpu/{}/specialties",
        base_url, hospital_id
    ))
    .await
    .unwrap()
    .text()
    .await
    .unwrap();

    let file = File::create(
        dir_name
            .join("specialties")
            .join(hospital_id.to_owned() + ".json"),
    )
    .await;

    if let Ok(mut file) = file {
        let _ = file.write_all(res.as_bytes()).await;
    }

    serde_json::from_str(&res).unwrap()
}

async fn fetch_doctors_for_specialty(
    hospital_id: String,
    specialty_id: String,
    base_url: String,
    dir_name: PathBuf,
) {
    let res = reqwest::get(format!(
        "{}/schedule/lpu/{}/speciality/{}/doctors",
        base_url, hospital_id, specialty_id
    ))
    .await
    .unwrap()
    .text()
    .await
    .unwrap();

    let file = File::create(
        dir_name
            .join("hospitals-specialties")
            .join(format!("{}-{}.json", hospital_id, specialty_id)),
    )
    .await;

    if let Ok(mut file) = file {
        let _ = file.write_all(res.as_bytes()).await;
    }
}

fn get_id_string(id_data: &Value) -> Result<String, ()> {
    match id_data {
        Value::String(id) => Ok(id.clone()),
        Value::Number(id) => Ok(id.to_string()),
        _ => Err(()),
    }
}
