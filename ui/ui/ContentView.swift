//
//  ContentView.swift
//  ui
//
//  Created by Michael Nitschinger on 25/11/2020.
//  Copyright © 2020 Michael Nitschinger. All rights reserved.
//

import SwiftUI

struct ContentView: View {
    var body: some View {
        TabView {
            DashboardView()
                .tabItem {
                    Image(systemName: "1.square.fill")
                    Text("Dashboard")
                }
            SettingsView()
                .tabItem {
                    Image(systemName: "2.square.fill")
                    Text("Settings")
                }
        }
        .font(.headline)
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
