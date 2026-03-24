/* IMMICH COVER PHOTOS ----------------------------------------------------------------------- */

async function check_immich_authorization() {
    const immichUrl = document.getElementById("immichUrl").textContent;
    try {
        const response = await fetch(immichUrl + "api/auth/status");
        //console.log("Immich Auth OK");
        if ( response.status === 200 ) { document.getElementById('dotImmich').className = 'status-dot green'; }
    } catch (err) {
        //document.getElementById("immich_authorization_status_NOT_OK").style.display = "block";
        //console.log("Immich Auth NOT OK");
        return false;
    }
}


async function get_cover_album_list(immichUrl, immichCoverAlbumId) {
    const response = await fetch(immichUrl + "api/albums/" + immichCoverAlbumId);

    if (!response.ok) {
        throw new Error("Failed to fetch album: " + response.status + " " + response.statusText);
    }

    return response.json();

}


async function add_photo_to_album(immichUrl, immichCoverAlbumId, assetIds) {
    const response = await fetch(immichUrl + "api/albums/" + immichCoverAlbumId + "/assets", {
        method: "PUT",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            ids: assetIds
        })
    });

    if (!response.ok) {
        const errorText = await response.text();
        throw new Error(
            "Failed to add assets to album: " +
            response.status + " " +
            response.statusText + " - " +
            errorText
        );
    }

    return response.json();
}

async function searchByOriginalPath(immichUrl, originalPath) {
    if (!originalPath?.trim()) return null;

    const response = await fetch(new URL("/api/search/metadata", immichUrl), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            originalPath: originalPath.normalize("NFC")
        })
    });

    return response.json();
}


async function searchMissingCoverPhotos(immichUrl, missingCoverPhotos) {
    const results = [];

    for (const item of missingCoverPhotos) {
        const originalPath = item.CoverPhoto;

        try {
            const searchResult = await searchByOriginalPath(
                immichUrl,
                originalPath
            );

            const assetId = searchResult?.assets?.items?.[0]?.id;
            if (assetId) {
                results.push({
                    coverPhoto: originalPath,
                    outerId: item.OuterId,
                    assetId
                });
            }
        } catch (err) {
            console.error(err);
        }
    }

    return results;
}


// RUN THIS TO SYNC THE IMMICH ALBUM
async function sync_album_covers(immichUrl, immichCoverAlbumId) {

    const cover_albums_db_text =
    document.getElementById("coverPhotoOriginalPaths").textContent;

    const cover_albums_db = JSON.parse(cover_albums_db_text);

    const cover_albums_response = await get_cover_album_list(
        immichUrl,
        immichCoverAlbumId
    );

    const existingOriginalPaths = new Set(
        cover_albums_response.assets.map(a => a.originalPath)
    );

    // Filter CoverPhoto entries that do NOT exist in the album assets
    const missingCoverPhotos = cover_albums_db.filter(item => {
        return !existingOriginalPaths.has(item.CoverPhoto);
    });

    console.log("Missing cover photos:", missingCoverPhotos);
    document.getElementById('count_missing_photos').textContent = missingCoverPhotos.length;

    const searchResults = await searchMissingCoverPhotos(
        immichUrl,
        missingCoverPhotos
    );

    const assetIds = searchResults.map(r => r.assetId).filter(id => id != null);

    const addPhotos = await add_photo_to_album(immichUrl, immichCoverAlbumId, assetIds);

}

// RUN THIS TO SAVE IMMICH ALBUM TO OPFS

async function save_album_covers_ids2OPFS(immichUrl, immichCoverAlbumId) {

    const cover_albums_db_text =
    document.getElementById("coverPhotoOriginalPaths").textContent;

    const cover_albums_db = JSON.parse(cover_albums_db_text);

    const cover_albums_response = await get_cover_album_list(
        immichUrl,
        immichCoverAlbumId
    );

    /*const outerIdToAssetId = Object.fromEntries(
        cover_albums_db
        .map(dbItem => {
            const asset = cover_albums_response.assets.find(
                a => a.originalPath === dbItem.CoverPhoto
            );
            return asset ? [dbItem.OuterId, asset.id] : null;
        })
        .filter(Boolean)
    );*/

    const outerIdToAssetId = Object.fromEntries(
    cover_albums_db
        .map(dbItem => {
            const coverPhoto = dbItem.CoverPhoto;

            if (coverPhoto.startsWith('!genre[') && coverPhoto.endsWith(']')) {
                const genreValue = coverPhoto.slice(7, -1);
                return [dbItem.OuterId, genreValue];
            }

            const asset = cover_albums_response.assets.find(
                a => a.originalPath === coverPhoto
            );
            return asset ? [dbItem.OuterId, immichUrl + "api/assets/" + asset.id + "/thumbnail"] : null;
        })
        .filter(Boolean)
    );

    await (await (await navigator.storage.getDirectory())
    .getFileHandle("cover_photos.json", { create: true }))
    .createWritable()
    .then(w => (w.write(JSON.stringify(outerIdToAssetId, null, 2)), w.close())).then(() => {
        document.getElementById("cover2opfs").textContent = "OK";
    }).then(()=>{location.reload()});

}

/* IMMICH ALBUMS ----------------------------------------------------------------------- */
async function get_album_id_from_name(immichUrl, albumName) {
    const response = await fetch(immichUrl + "api/albums");

    if (!response.ok) {
        throw new Error("Failed to fetch albums: " + response.status + " " + response.statusText);
    }

    const albums = await response.json();

    const album = albums.find(a => a.albumName === albumName);

    if (!album) {
        alert("No album with that name found");
        return null;
    }

    return album.id;
}

async function process_album_string(immichUrl, input) {
    const regex = /\[([^\]]+)\]\(([\s\S]*?)\)/g; // allow multiline $2

    const UUID_REGEX =
    /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

    let match;
    while ((match = regex.exec(input)) !== null) {
        const albumName = match[1]; // $1
        const filesPart = match[2]; // $2 (multiline)

        // Resolve album ID
        const immichAlbumId = await get_album_id_from_name(immichUrl, albumName);
        if (!immichAlbumId) {
            continue; // alert already shown
        }

        // Split + dedupe file paths
        const filenames = Array.from(
            new Set(
                filesPart
                .trim()
                .split(/\s+/)
                .filter(Boolean)
            )
        );

        const assetIds = [];

        for (const filename of filenames) {
            try {
                const result = await searchByOriginalPath(immichUrl, filename);

                if (result.assets?.total === 0) {
                    console.warn("No asset found for path:", filename);
                    continue;
                }

                // Normalize possible return shapes from searchByOriginalPath
                let assetId = null;

                if (typeof result === "string") {
                    assetId = result;
                }
                else if (Array.isArray(result) && result.length > 0) {
                    assetId = result[0]?.id ?? result[0];
                }
                else if (result && typeof result === "object") {
                    // Immich search response shape
                    if (result.assets?.items?.length > 0) {
                        assetId = result.assets.items[0].id;
                    }
                    // Fallback: generic object with id
                    else if (result.id) {
                        assetId = result.id;
                    }
                }

                if (!assetId || !UUID_REGEX.test(assetId)) {
                    console.warn(
                        "Rejected non-UUID assetId for path:",
                        filename,
                        "→",
                        result
                    );
                    continue;
                }

                assetIds.push(assetId);

            } catch (err) {
                console.error("Failed to resolve asset for:", filename, err);
            }
        }

        if (assetIds.length === 0) {
            console.warn("No valid assets found for album:", albumName);
            continue;
        }

        // Add assets to album
        await add_photo_to_album(immichUrl, immichAlbumId, assetIds);
    }
}

// Plot Photos to trip map
// await fetchAllMediaInInterval({startDateTime: "2025-01-01T00:00:00.000Z", endDateTime: "2025-01-15T23:59:59.999Z"})
async function fetchAllMediaInInterval({startDateTime, endDateTime}) {
  const size = 1000;
  let page = 1;
  let all = [];
  const immichUrl = document.getElementById("immichUrl").textContent;
  while (true) {
      const res = await fetch(`${immichUrl}api/search/metadata`, {
      method: "POST", headers: { "Content-Type": "application/json"
      },
      body: JSON.stringify({ takenAfter: startDateTime, takenBefore: endDateTime, page, size, withExif: true },
    )
    });
    if (!res.ok) { throw new Error(`HTTP ${res.status} on page ${page}`);
    }
    const data = await res.json();
    const items = data.assets?.items ?? [];
    if (items.length === 0) break;
    all.push(...items);
    // stop early if last page
    if (items.length < size) break; page++;
  }
  return all;
}

async function fetchAllMediaWithCoords() {

    const startDateTime = document.getElementById("startDateTime").textContent;
    const endDateTime = document.getElementById("endDateTime").textContent;

    const config = { startDateTime: startDateTime, endDateTime: endDateTime }

    const map = initiate_map();
    if (!map) return;

    if (!map.isStyleLoaded()) {
        await new Promise(res => map.once("load", res));
    }

    const all = await fetchAllMediaInInterval(config);

    const assets = all
    .map(a => {
        const lat = Number(a.exifInfo?.latitude);
        const lon = Number(a.exifInfo?.longitude);
        if (
            !Number.isFinite(lat) ||
            !Number.isFinite(lon) ||
            lat < -90 || lat > 90 ||
            lon < -180 || lon > 180 ||
            (lat === 0 && lon === 0)
        ) return null;

        return {
            id: a.id,
            type: a.type,
            lat,
            lon,
            date: a.fileCreatedAt ?? a.localDateTime ?? null,
        };
    })
    .filter(Boolean);

    if (!assets.length) return assets;

    const geojson = {
        type: "FeatureCollection",
        features: assets.map(a => ({
            type: "Feature",
            geometry: {
                type: "Point",
                coordinates: [a.lon, a.lat],
            },
            properties: a,
        })),
    };

    const lineGeoJSON = {
        type: "Feature",
        geometry: {
            type: "LineString",
            coordinates: assets.map(a => [a.lon, a.lat]),
        },
    };

    if (map.getSource("media")) {
        map.getSource("media").setData(geojson);
    } else {
        map.addSource("media", {
            type: "geojson",
            data: geojson,
        });

        map.addLayer({
            id: "media-points",
            type: "circle",
            source: "media",
            paint: {
                'circle-radius': 6,
                'circle-color': 'red',
                'circle-stroke-color': '#000',
                'circle-stroke-width': 1,
                'circle-opacity': 0.7,
            },
        });

        // 👉 Popup handler (only register once)
        map.on("click", "media-points", (e) => {
            const f = e.features[0];
            const { id, date, type } = f.properties;

            new maplibregl.Popup()
            .setLngLat(f.geometry.coordinates)
            .setHTML(`
            <b>${type}</b><br>
            ${date ?? ""}<br>
            ${id}
            `)
            .addTo(map);
        });

        map.on("mouseenter", "media-points", () => {
            map.getCanvas().style.cursor = "pointer";
        });

        map.on("mouseleave", "media-points", () => {
            map.getCanvas().style.cursor = "";
        });
    }

    if (map.getSource("media-line")) {
        map.getSource("media-line").setData(lineGeoJSON);
    } else {
        map.addSource("media-line", {
            type: "geojson",
            data: lineGeoJSON,
        });

        map.addLayer({
            id: "media-line-layer",
            type: "line",
            source: "media-line",
            layout: {
                "line-cap": "round",
                "line-join": "round",
            },
            paint: {
                "line-color": "#ff5500",
                "line-width": 3,
                "line-opacity": 0.7,
            },
        });
    }

    const bounds = new maplibregl.LngLatBounds();
    assets.forEach(a => bounds.extend([a.lon, a.lat]));
    map.fitBounds(bounds, { padding: 40, maxZoom: 12 });

    return assets;
}
