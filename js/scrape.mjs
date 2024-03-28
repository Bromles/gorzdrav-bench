import fs from "node:fs/promises";

const dirName = "./mockData";
const baseUrl = "https://gorzdrav.spb.ru/_api/api/v2";

await fs.access(dirName).catch(() => fs.mkdir(dirName));

const hospitalsRes = await fetch(`${baseUrl}/shared/lpus`);
const hospitals = await hospitalsRes.json();

await fs.writeFile(
  `${dirName}/hospitals.json`,
  JSON.stringify(hospitals, null, 2)
);

const districtsRes = await fetch(`${baseUrl}/shared/districts`);
const districts = await districtsRes.json();

await fs.writeFile(
  `${dirName}/districts.json`,
  JSON.stringify(districts, null, 2)
);

await fs
  .access(`${dirName}/specialties`)
  .catch(() => fs.mkdir(`${dirName}/specialties`));

await fs
  .access(`${dirName}/hospitals-specialties`)
  .catch(() => fs.mkdir(`${dirName}/hospitals-specialties`));

if (hospitals.result) {
  for (const hospital of hospitals.result) {
    const specialtiesRes = await fetch(
      `${baseUrl}/schedule/lpu/${hospital.id}/specialties`
    );

    specialtiesRes
      .json()
      .then(async (specialties) => {
        await fs.writeFile(
          `${dirName}/specialties/${hospital.id}.json`,
          JSON.stringify(specialties, null, 2)
        );

        if (specialties.result) {
          await Promise.allSettled(
            specialties.result.map(async (specialty) => {
              const doctorsRes = await fetch(
                `${baseUrl}/schedule/lpu/${hospital.id}/speciality/${specialty.id}/doctors`
              );

              doctorsRes
                .json()
                .then(async (doctors) => {
                  await fs.writeFile(
                    `${dirName}/hospitals-specialties/${hospital.id}-${specialty.id}.json`,
                    JSON.stringify(doctors, null, 2)
                  );
                })
                .catch(() => undefined);
            })
          );
        }
      })
      .catch(() => undefined);
  }
}
