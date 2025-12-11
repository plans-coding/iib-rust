SELECT
    'EventQuery' AS Query,
    *
FROM
    IIBb_Events
WHERE
(
    COALESCE(OuterId, '') || ' ' || COALESCE(OverallDestination, '') || ' ' || COALESCE(Date, '') || ' ' || COALESCE(Events, '') || ' ' || COALESCE(Accommodation, '') || ' ' || COALESCE(AccommodationCountry, '') || ' ' || COALESCE(ParticipantGroup, '') || ' ' || COALESCE(TravelParticipants, '') || ' ' || COALESCE(AdditionalNotes, '') || ' ' || COALESCE(CountriesDuringDay, '')
) LIKE '%${parameter}%';