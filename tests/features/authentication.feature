Feature: Authentication
  As an API consumer
  I want to authenticate using valid credentials
  So that I can access protected endpoints

  Background:
    Given an admin account "Bootstrap Admin" with email "admin@webrust.dev" and password "ChangeMe123!"

  Scenario: Successful authentication
    When I authenticate with email "admin@webrust.dev" and password "ChangeMe123!"
    Then the authentication succeeds
    And the returned user role is "admin"
    And the access token is issued

  Scenario: Authentication fails with wrong password
    When I authenticate with email "admin@webrust.dev" and password "WrongPass123!"
    Then the authentication fails with message "invalid credentials"
    And no access token is issued