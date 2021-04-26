import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/adaptive.dart';
import 'package:esse/widgets/list_system_app.dart';
import 'package:esse/options.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/service/list.dart';
import 'package:esse/apps/service/models.dart';
import 'package:esse/apps/chat/provider.dart';
import 'package:esse/apps/chat/list.dart';

class DefaultHomeShow extends StatelessWidget {
  const DefaultHomeShow({Key key}): super(key: key);

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final lang = AppLocalizations.of(context);
    final chatProvider = context.watch<ChatProvider>();
    final chatTops = chatProvider.topKeys;
    final friends = chatProvider.friends;

    return Column(children: [
        ListSystemApp(name: lang.chats, icon: Icons.people_rounded,
          callback: () => Provider.of<AccountProvider>(context, listen: false).updateActivedApp(
            null, lang.chats, ChatList())),
        ListSystemApp(name: lang.groups, icon: Icons.grid_view_rounded,
          callback: () => Provider.of<AccountProvider>(context, listen: false).updateActivedApp(
            null, lang.chats, ServiceList())),
        const SizedBox(height: 5.0),
        const Divider(height: 1.0, color: Color(0x40ADB0BB)),
        const SizedBox(height: 5.0),
        Column(
          children: INNER_SERVICES.map((v) {
              final params = v.params(lang);
              return ListInnerService(
                name: params[0],
                bio: params[1],
                logo: params[2],
                callback: () => v.callback(context, isDesktop, lang),
                isDesktop: isDesktop,
              );
          }).toList()
        ),
        Expanded(
          child: ListView.builder(
            itemCount: chatTops.length,
            itemBuilder: (BuildContext ctx, int index) => ListChat(
              friend: friends[chatTops.keys.elementAt(index)]),
          ),
        )
    ]);
  }
}
