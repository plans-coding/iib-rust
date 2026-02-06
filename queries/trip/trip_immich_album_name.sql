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

SELECT substr(PhotoAlbums, instr(PhotoAlbums, '[') + 1, instr(PhotoAlbums, ']') - instr(PhotoAlbums, '[') - 1) AS ImmichAlbumName FROM Overview WHERE OuterId = '_OUTER_ID_' AND PhotoAlbums NOT LIKE '!desc[%]%' AND PhotoAlbums LIKE '%[%]%';
