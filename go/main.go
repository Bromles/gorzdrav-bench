package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
)

var dirName = filepath.Join(".", "mockData")

const baseUrl = "https://gorzdrav.spb.ru/_api/api/v2"

func main() {
	os.MkdirAll(dirName, os.ModePerm)

	hospitalsData := fetchHospitals()
	fetchDistricts()

	os.MkdirAll(filepath.Join(dirName, "specialties"), os.ModePerm)
	os.MkdirAll(filepath.Join(dirName, "hospitals-specialties"), os.ModePerm)

	fetchSpecialties(hospitalsData)
}

func readBody(res *http.Response) []byte {
	defer res.Body.Close()

	bytes, err := io.ReadAll(res.Body)
	if err != nil {
		panic("Cannot convert res to bytes")
	}

	return bytes
}

func fetchHospitals() map[string]any {
	hospitalsRes, _ := http.Get(baseUrl + "/shared/lpus")

	var hospitals map[string]any
	bytes := readBody(hospitalsRes)

	json.Unmarshal(bytes, &hospitals)

	os.WriteFile(filepath.Join(dirName, "hospitals.json"), bytes, os.ModePerm)

	return hospitals
}

func fetchDistricts() {
	districtsRes, _ := http.Get(baseUrl + "/shared/districts")

	var districts map[string]any
	bytes := readBody(districtsRes)
	json.Unmarshal(bytes, &districts)

	os.WriteFile(filepath.Join(dirName, "districts.json"), bytes, os.ModePerm)
}

func fetchSpecialties(hospitalsData map[string]any) {
	if hospitals, found := hospitalsData["result"]; found {
		hospitals, ok := hospitals.([]any)
		if !ok {
			return
		}

		for _, hospital := range hospitals {
			hospital, ok := hospital.(map[string]any)
			if !ok {
				continue
			}

			if hospitalId, found := hospital["id"]; found {
				hospitalId, ok := getIdString(hospitalId)
				if !ok {
					continue
				}

				specialtiesData := fetchSpecialtiesForHospital(hospitalId)

				if specialties, found := specialtiesData["result"]; found {
					specialties, ok := specialties.([]any)
					if !ok {
						continue
					}

					for _, specialty := range specialties {
						specialty, ok := specialty.(map[string]any)
						if !ok {
							continue
						}

						if specialtyId, found := specialty["id"]; found {
							specialtyId, ok := getIdString(specialtyId)
							if !ok {
								continue
							}

							go fetchDoctorsForSpecialty(hospitalId, specialtyId)
						}
					}
				}
			}
		}

	}
}

func fetchSpecialtiesForHospital(hospitalId string) map[string]any {
	specialtiesRes, _ := http.Get(baseUrl + "/schedule/lpu/" + hospitalId + "/specialties")

	var specialties map[string]any
	bytes := readBody(specialtiesRes)
	json.Unmarshal(bytes, &specialties)

	os.WriteFile(filepath.Join(dirName, "specialties", hospitalId+".json"), bytes, os.ModePerm)

	return specialties
}

func fetchDoctorsForSpecialty(hospitalId string, specialtyId string) {
	doctorsRes, err := http.Get(baseUrl + "/schedule/lpu/" + hospitalId + "/speciality/" + specialtyId + "/doctors")

	if err != nil {
		return
	}

	var doctors map[string]any
	bytes := readBody(doctorsRes)
	err = json.Unmarshal(bytes, &doctors)

	if err != nil {
		return
	}

	os.WriteFile(filepath.Join(dirName, "hospitals-specialties", hospitalId+"-"+specialtyId+".json"), bytes, os.ModePerm)
}

func getIdString(idData any) (string, bool) {
	switch id := idData.(type) {
	case string:
		return id, true
	case float64:
		return fmt.Sprintf("%g", id), true
	default:
		return "", false
	}
}
