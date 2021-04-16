import 'package:flutter/material.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/models/friend.dart';
import 'package:esse/widgets/shadow_dialog.dart';

openContact(BuildContext context, ColorScheme color, AppLocalizations lang, List<int> orders, Map<int, Friend> friends, Function callback) {
  showShadowDialog(
    context, Icons.person_rounded, lang.contact, Column(
      children: [
        Container(
          height: 40.0,
          decoration: BoxDecoration(
            color: color.surface,
            borderRadius: BorderRadius.circular(15.0)),
          child: TextField(
            autofocus: false,
            textInputAction: TextInputAction.search,
            textAlignVertical: TextAlignVertical.center,
            style: TextStyle(fontSize: 14.0),
            onSubmitted: (value) {
              //toast(context, 'WIP...');
            },
            decoration: InputDecoration(
              hintText: lang.search,
              hintStyle: TextStyle(color: color.onPrimary.withOpacity(0.5)),
              border: InputBorder.none,
              contentPadding: EdgeInsets.only(left: 15.0, right: 15.0, bottom: 15.0),
            ),
          ),
        ),
        SizedBox(height: 15.0),
        Column(
          children: orders.map<Widget>((id) {
              final contact = friends[id];
              return GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => callback(context, contact.id),
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 14.0),
                  child: Row(
                    children: [
                      contact.showAvatar(needOnline: false),
                      SizedBox(width: 15.0),
                      Text(contact.name, style: TextStyle(fontSize: 16.0)),
                    ],
                  ),
                ),
              );
          }).toList()
        )
      ]
    )
  );

}
