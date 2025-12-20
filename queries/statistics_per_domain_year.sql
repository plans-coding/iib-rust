SELECT
	SUBSTR(o.InnerId, 1, 1) AS TripDomain,
    SUBSTR(o.InnerId, 1, 1) AS DomainAbbreviation,
    SUBSTR(o.DepartureDate, 1, 4) AS Year,
    COUNT(*) AS AbbreviationCount,
    d.DomainDescription,
	ParticipantGroup
FROM
    bewa_Overview o
LEFT JOIN
    bewx_TripDomains d
ON
    SUBSTR(o.InnerId, 1, 1) = d.DomainAbbreviation
WHERE
    o.InnerId IS NOT NULL
    AND o.DepartureDate IS NOT NULL
	/*AND TripDomain IN (TripDomain) AND ParticipantGroup IN (ParticipantGroup)*/
GROUP BY
    DomainAbbreviation, Year
ORDER BY
    Year ASC, AbbreviationCount DESC;