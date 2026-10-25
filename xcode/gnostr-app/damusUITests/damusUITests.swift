//
//  damusUITests.swift
//  damusUITests
//
//  Created by William Casarin on 2022-04-01.
//

import XCTest

class damusUITests: XCTestCase {

    override func setUpWithError() throws {
        // Put setup code here. This method is called before the invocation of each test method in the class.

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false

        // In UI tests it’s important to set the initial state - such as interface orientation - required for your tests before they run. The setUp method is a good place to do this.
    }

    override func tearDownWithError() throws {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }

    func testExample() throws {
        // UI tests must launch the application that they test.
        let app = XCUIApplication()
        app.launch()

        // Use XCTAssert and related functions to verify your tests produce the correct results.
    }

    func testGlobalNIP34HeaderKeepsSettingsBesideToggleAcrossNavigation() {
        let app = XCUIApplication()
        app.launchArguments = [
            "-pubkey", "3efdaebb1d8923ebd99c9e7ace3b4194ab45512e2be79c1b7d68d9243e0d2681",
            "-nip34_only", "NO",
            "-auto_subscribe_nip34", "NO",
        ]
        app.launch()
        let search = app.textFields["global-search"]
        let toggle = app.switches["NIP-34 only"]
        let settings = app.buttons["global-settings"]
        XCTAssertTrue(toggle.waitForExistence(timeout: 15))
        XCTAssertTrue(settings.isHittable)
        XCTAssertTrue(search.isHittable)
        XCTAssertLessThan(search.frame.midX, toggle.frame.midX)
        XCTAssertGreaterThan(settings.frame.midX, toggle.frame.midX)
        XCTAssertLessThan(abs(settings.frame.midY - toggle.frame.midY), 20)
        toggle.coordinate(withNormalizedOffset: CGVector(dx: 0.95, dy: 0.5)).tap()
        XCTAssertEqual(toggle.value as? String, "1")
        let repositories = app.buttons["nip34-kind-30617"]
        XCTAssertTrue(repositories.waitForExistence(timeout: 5))
        if repositories.value as? String == "Off" {
            repositories.tap()
        }
        XCTAssertEqual(repositories.value as? String, "On")
        repositories.tap()
        XCTAssertEqual(repositories.value as? String, "Off")

        settings.tap()
        XCTAssertTrue(app.navigationBars["Settings"].waitForExistence(timeout: 5))

        search.tap()
        search.typeText("#gnostr\n")
        let hashtag = app.buttons["Search hashtag: #gnostr"]
        XCTAssertTrue(hashtag.waitForExistence(timeout: 5))
        XCTAssertEqual(search.value as? String, "#gnostr")
        XCTAssertEqual(repositories.value as? String, "Off")
        hashtag.tap()
        XCTAssertTrue(app.navigationBars["#gnostr"].waitForExistence(timeout: 5))
        XCTAssertTrue(search.isHittable)
        XCTAssertTrue(toggle.isHittable)
        XCTAssertTrue(settings.isHittable)
        XCTAssertEqual(app.switches.matching(identifier: "NIP-34 only").count, 1)
        XCTAssertTrue(toggle.isHittable)
        XCTAssertTrue(settings.isHittable)
        app.navigationBars["Settings"].buttons.firstMatch.tap()
        XCTAssertEqual(repositories.value as? String, "Off")

        app.buttons["Your profile"].tap()
        XCTAssertTrue(app.staticTexts["Followers"].waitForExistence(timeout: 5))
        XCTAssertTrue(toggle.isHittable)
        settings.tap()
        XCTAssertTrue(app.navigationBars["Settings"].waitForExistence(timeout: 5))
    }

    func testLaunchPerformance() throws {
        if #available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 7.0, *) {
            // This measures how long it takes to launch your application.
            measure(metrics: [XCTApplicationLaunchMetric()]) {
                XCUIApplication().launch()
            }
        }
    }
}
