WITH Overview AS (
    SELECT
        substr(TripDomain,1,1) ||
        substr(ParticipantGroup,1,1) ||
        printf('%03d',
            ROW_NUMBER() OVER (
                PARTITION BY substr(TripDomain,1,1), substr(ParticipantGroup,1,1)
                ORDER BY DepartureDate
            )
        ) AS OuterId,
        *
    FROM bewa_Overview
)

SELECT CASE WHEN PhotoAlbums LIKE '!desc[%]%' THEN substr(PhotoAlbums, instr(PhotoAlbums, '[') + 1, instr(PhotoAlbums, ']') - instr(PhotoAlbums, '[') - 1) ELSE NULL END AS ImmichDescSearch FROM Overview WHERE OuterId = '_OUTER_ID_';
