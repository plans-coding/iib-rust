SELECT
    COUNT(InnerId) AS Count,
    SUBSTR(InnerId, 1, 1) AS TripDomain
FROM
    bewa_Overview
/*WHERE
    SUBSTR(InnerId, 1, 1) IN (TripDomain)
    AND ParticipantGroup IN (ParticipantGroup)*/;
