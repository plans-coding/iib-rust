SELECT
    *,
    CAST(strftime('%Y', DepartureDate) AS TEXT) AS TripYear,
    CAST(
    julianday(ReturnDate) - julianday(DepartureDate)
    AS INTEGER
) AS NumberOfDays
FROM bewa_Overview
WHERE
    OuterId IS NOT NULL
    AND TripDomain IN (TripDomain)
    AND ParticipantGroup IN (ParticipantGroup) AND 1=1;
