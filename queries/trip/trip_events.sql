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

SELECT
	   o.OuterId,
	   e.InnerId,
       o.OverallDestination AS OverallDestination,
	   e.Date AS Date,
	   e.Events AS Events,
	   e.Accommodation AS Accommodation,
	   e.AccommodationCountry AS AccommodationCountry,
	   e.AccommodationCoordinatesAccuracy AS AccommodationCoordinatesAccuracy,
	   e.AccommodationCoordinates AS AccommodationCoordinates,
       o.ParticipantGroup AS ParticipantGroup,
       e.TravelParticipants AS TravelParticipants,
	   e.AdditionalNotes AS AdditionalNotes,
	   e.CountriesDuringDay AS CountriesDuringDay
FROM bewb_Events e
JOIN Overview o ON e.InnerId = o.InnerId
WHERE
    o.OuterId = '/*_OUTER_ID_*/'
ORDER BY Date ASC;
