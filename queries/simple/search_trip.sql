SELECT
    'TripQuery' AS Query,
    *
FROM
    bewa_Overview
RIGHT JOIN IIBc_BorderCrossings USING (InnerId, OuterId)
WHERE
(
    COALESCE(ParticipantGroup, '') || ' ' || COALESCE(OverallDestination, '') || ' ' || COALESCE(DepartureDate, '') || ' ' || COALESCE(ReturnDate, '') || ' ' || COALESCE(MapPins, '') || ' ' || COALESCE(TripDescription, '') || ' ' || COALESCE(AllBorderCrossings, '')
) LIKE '%/*_STRING_*/%';