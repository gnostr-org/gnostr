//===----------------------------------------------------------------------===//
//
// This source file is part of the swift-libp2p open source project
//
// Copyright (c) 2022-2025 swift-libp2p project authors
// Licensed under MIT
//
// See LICENSE for license information
// See CONTRIBUTORS for the list of swift-libp2p project authors
//
// SPDX-License-Identifier: MIT
//
//===----------------------------------------------------------------------===//
//
//  Created by Vapor
//  Modified by swift-libp2p
//

import Foundation
import LibP2PTesting
import Queues
import Testing

@Suite("Schedule Builder Tests")
struct ScheduleBuilderTests {

    init() async throws {
        try #require(isLoggingConfigured)
    }

    @Test func testScheduleBuilderAPI() async throws {
        func configure(_ app: Application) async throws {
            app.queues.use(.test)
        }

        try await withApp(configure: configure) { app in
            // yearly
            app.queues.schedule(Cleanup()).yearly().in(.may).on(23).at(.noon)

            // monthly
            app.queues.schedule(Cleanup()).monthly().on(15).at(.midnight)

            // weekly
            app.queues.schedule(Cleanup()).weekly().on(.monday).at("3:13am")

            // daily
            app.queues.schedule(Cleanup()).daily().at("5:23pm")

            // daily 2
            app.queues.schedule(Cleanup()).daily().at(5, 23, .pm)

            // daily 3
            app.queues.schedule(Cleanup()).daily().at(17, 23)

            // hourly
            app.queues.schedule(Cleanup()).hourly().at(30)
        }
    }

    @Test func testHourlyBuilder() throws {
        let builder = ScheduleBuilder()
        builder.hourly().at(30)
        // same time
        #expect(
            builder.nextDate(current: Date(hour: 5, minute: 30))
                // plus one hour
                == Date(hour: 6, minute: 30)
        )
        // just before
        #expect(
            builder.nextDate(current: Date(hour: 5, minute: 29))
                // plus one minute
                == Date(hour: 5, minute: 30)
        )
        // just after
        #expect(
            builder.nextDate(current: Date(hour: 5, minute: 31))
                // plus one hour
                == Date(hour: 6, minute: 30)
        )
    }

    @Test func testDailyBuilder() throws {
        let builder = ScheduleBuilder()
        builder.daily().at("5:23am")
        // same time
        #expect(
            builder.nextDate(current: Date(day: 1, hour: 5, minute: 23))
                // plus one day
                == Date(day: 2, hour: 5, minute: 23)
        )
        // just before
        #expect(
            builder.nextDate(current: Date(day: 1, hour: 5, minute: 22))
                // plus one minute
                == Date(day: 1, hour: 5, minute: 23)
        )
        // just after
        #expect(
            builder.nextDate(current: Date(day: 1, hour: 5, minute: 24))
                // plus one day
                == Date(day: 2, hour: 5, minute: 23)
        )
    }

    @Test func testWeeklyBuilder() throws {
        let builder = ScheduleBuilder()
        builder.weekly().on(.monday).at(.noon)
        // sunday before
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 1, day: 6, hour: 5, minute: 23))
                // next day at noon
                == Date(year: 2019, month: 1, day: 7, hour: 12, minute: 00)
        )
        // monday at 1pm
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 1, day: 7, hour: 13, minute: 00))
                // next monday at noon
                == Date(year: 2019, month: 1, day: 14, hour: 12, minute: 00)
        )
        // monday at 11:30am
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 1, day: 7, hour: 11, minute: 30))
                // same day at noon
                == Date(year: 2019, month: 1, day: 7, hour: 12, minute: 00)
        )
    }

    @Test func testMonthlyBuilderFirstDay() throws {
        let builder = ScheduleBuilder()
        builder.monthly().on(.first).at(.noon)
        // middle of jan
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 1, day: 15, hour: 5, minute: 23))
                // first of feb
                == Date(year: 2019, month: 2, day: 1, hour: 12, minute: 00)
        )
        // just before
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 2, day: 1, hour: 11, minute: 30))
                // first of feb
                == Date(year: 2019, month: 2, day: 1, hour: 12, minute: 00)
        )
        // just after
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 2, day: 1, hour: 12, minute: 30))
                // first of feb
                == Date(year: 2019, month: 3, day: 1, hour: 12, minute: 00)
        )
    }

    @Test func testMonthlyBuilder15th() throws {
        let builder = ScheduleBuilder()
        builder.monthly().on(15).at(.noon)
        // just before
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 2, day: 15, hour: 11, minute: 30))
                // first of feb
                == Date(year: 2019, month: 2, day: 15, hour: 12, minute: 00)
        )
        // just after
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 2, day: 15, hour: 12, minute: 30))
                // first of feb
                == Date(year: 2019, month: 3, day: 15, hour: 12, minute: 00)
        )
    }

    @Test func testYearlyBuilder() throws {
        let builder = ScheduleBuilder()
        builder.yearly().in(.may).on(23).at("2:58pm")
        // early in the year
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 1, day: 15, hour: 5, minute: 23))
                // 2019
                == Date(year: 2019, month: 5, day: 23, hour: 14, minute: 58)
        )
        // just before
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 5, day: 23, hour: 14, minute: 57))
                // one minute later
                == Date(year: 2019, month: 5, day: 23, hour: 14, minute: 58)
        )
        // just after
        #expect(
            builder.nextDate(current: Date(year: 2019, month: 5, day: 23, hour: 14, minute: 59))
                // one year later
                == Date(year: 2020, month: 5, day: 23, hour: 14, minute: 58)
        )
    }

    @Test func testCustomCalendarBuilder() throws {
        let est = Calendar.calendar(timezone: "EST")
        let mst = Calendar.calendar(timezone: "MST")

        // Create a date at 8:00pm EST
        let estDate = Date(calendar: est, hour: 20, minute: 00)

        // Schedule it for 7:00pm MST
        let builder = ScheduleBuilder(calendar: mst)
        builder.daily().at("7:00pm")

        #expect(
            builder.nextDate(current: estDate)
                // one hour later
                == Date(calendar: est, hour: 21, minute: 00)
        )
    }

}

final class Cleanup: ScheduledJob {
    func run(context: QueueContext) -> EventLoopFuture<Void> {
        context.eventLoop.makeSucceededVoidFuture()
    }
}

extension Date {
    init(
        calendar: Calendar = .current,
        year: Int = 2020,
        month: Int = 1,
        day: Int = 1,
        hour: Int = 0,
        minute: Int = 0,
        second: Int = 0
    ) {
        self = DateComponents(
            calendar: calendar,
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second
        ).date!
    }
}

extension Calendar {
    fileprivate static func calendar(timezone identifier: String) -> Calendar {
        var calendar = Calendar.current
        calendar.timeZone = TimeZone(identifier: identifier)!
        return calendar
    }
}
