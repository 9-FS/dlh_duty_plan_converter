# dlh_duty_plan_converter
## Introduction

This program regurarily downloads the DLH myTime duty plan from `INPUT_CALENDAR_URL`, classifies events according to their summary (title), converts them into a preferred format, and saves the output calendar in `./calendar/duty_plan.ics`. Converting to the preferred format includes converting IATA codes to ICAO codes and full names, which is the reason why a database is connected. Depending on the duty plan event type, alarms are also set.