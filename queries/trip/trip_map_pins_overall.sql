WITH RECURSIVE Overview AS (
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
),
SplitPins AS
    (
        -- Start by selecting the first portion of the string
        SELECT
            OuterId,
            SUBSTR(MapPins, INSTR(MapPins, '[') + 2, INSTR(MapPins, ')') - INSTR(MapPins, '[') - 2) AS PinEntry,
            SUBSTR(MapPins, INSTR(MapPins, ')') + 2) AS Remaining
        FROM
            Overview
        /*WHERE
            OuterId = "_OUTER_ID_"*/
        UNION ALL
        -- Recursively extract subsequent portions
        SELECT
            OuterId,
            SUBSTR(Remaining, INSTR(Remaining, '[') + 2, INSTR(Remaining, ')') - INSTR(Remaining, '[') - 2) AS PinEntry,
            SUBSTR(Remaining, INSTR(Remaining, ')') + 2)
        FROM
            SplitPins
        WHERE
            Remaining LIKE '%{%'
    )
    -- Extract name and coordinates from each pin entry
SELECT
    OuterId,
    TRIM(SUBSTR(PinEntry, 1, INSTR(PinEntry, '](') - 1)) AS MapPin,
    TRIM(SUBSTR(PinEntry, INSTR(PinEntry, '](') + 2, INSTR(PinEntry, ',') - INSTR(PinEntry, '](') - 2)) AS Latitude,
    TRIM(SUBSTR(PinEntry, INSTR(PinEntry, ',') + 2)) AS Longitude
FROM
    SplitPins;
