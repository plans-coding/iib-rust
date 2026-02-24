SELECT
    DISTINCT AccommodationCountry
FROM
    bewb_Events e
JOIN bewa_Overview o ON e.InnerId = o.InnerId
WHERE
    AccommodationCountry NOT LIKE '(%'
    AND AccommodationCountry NOT LIKE '-%'
    AND TripDomain IN (TripDomain) AND ParticipantGroup IN (ParticipantGroup) AND 1=1
ORDER BY
    AccommodationCountry;
