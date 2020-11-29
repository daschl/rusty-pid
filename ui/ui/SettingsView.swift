//
//  SettingsView.swift
//  ui
//
//  Created by Michael Nitschinger on 25/11/2020.
//  Copyright © 2020 Michael Nitschinger. All rights reserved.
//

import SwiftUI

struct SettingsView: View {
    var body: some View {
        NavigationView {
                   List {
                       Text("Chocolate")
                       Text("Vanilla")
                       Text("Strawberry")
                   }
                   .navigationBarTitle(Text("Today‘s Flavors"))
                   .navigationBarItems(leading:
                       HStack {
                           Button("Hours") {
                               print("Hours tapped!")
                           }
                       }, trailing:
                       HStack {
                           Button("Favorites") {
                               print("Favorites tapped!")
                           }


                           Button("Specials") {
                               print("Specials tapped!")
                           }
                       }
                   )
               }
    }
}

struct SettingsView_Previews: PreviewProvider {
    static var previews: some View {
        SettingsView()
    }
}
