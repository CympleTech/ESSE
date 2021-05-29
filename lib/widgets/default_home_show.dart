import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/adaptive.dart';
import 'package:esse/widgets/list_system_app.dart';
import 'package:esse/options.dart';
import 'package:esse/provider.dart';
import 'package:esse/session.dart';

import 'package:esse/apps/chat/detail.dart';
import 'package:esse/apps/chat/provider.dart';
import 'package:esse/apps/group_chat/detail.dart';
import 'package:esse/apps/group_chat/provider.dart';
import 'package:esse/apps/assistant/page.dart';
import 'package:esse/apps/file/page.dart';

class DefaultHomeShow extends StatelessWidget {
  const DefaultHomeShow({Key key}): super(key: key);

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final lang = AppLocalizations.of(context);
    final provider = context.watch<AccountProvider>();
    final allKeys = provider.topKeys + provider.orderKeys;
    final sessions = provider.sessions;

    return ListView.builder(
      itemCount: allKeys.length,
      itemBuilder: (BuildContext ctx, int index) => _SessionWidget(
        session: sessions[allKeys[index]]
      ),
    );
  }
}

class _SessionWidget extends StatelessWidget {
  final Session session;
  const _SessionWidget({Key key, this.session}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);
    final params = session.parse(lang);

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () {
        String listTitle = "";
        Widget listWidget = null;
        Widget coreWidget = null;

        switch (session.type) {
          case SessionType.Chat:
            context.read<ChatProvider>().updateActivedFriend(session.fid);
            coreWidget = ChatDetail();
            break;
          case SessionType.Group:
            context.read<GroupChatProvider>().updateActivedGroup(session.fid);
            coreWidget = GroupChatDetail();
            break;
          case SessionType.Assistant:
            coreWidget = AssistantDetail();
            break;
          case SessionType.Files:
            listTitle = lang.files;
            listWidget = FolderList();
            break;
        }

        context.read<AccountProvider>().updateActivedSession(session.id);

        if (!isDesktop && coreWidget != null) {
          Navigator.push(context, MaterialPageRoute(builder: (_) => coreWidget));
        } else {
          context.read<AccountProvider>().updateActivedWidget(coreWidget, listTitle, listWidget);
        }
      },
      child: Container(
        height: 55.0,
        child: Row(
          children: [
            Container(
              width: 45.0,
              height: 45.0,
              margin: const EdgeInsets.only(left: 20.0, right: 15.0),
              child: params[0],
            ),
            Expanded(
              child: Container(
                height: 55.0,
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Expanded(
                          child: Text(params[1],
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(fontSize: 16.0))
                        ),
                        Container(
                          margin: const EdgeInsets.only(left: 15.0, right: 20.0),
                          child: Text(params[3],
                            style: const TextStyle(color: Color(0xFFADB0BB), fontSize: 12.0),
                          ),
                        )
                    ]),
                    const SizedBox(height: 4.0),
                    Row(
                      children: [
                        Expanded(
                          child: Text(params[2],
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: const TextStyle(color: Color(0xFFADB0BB), fontSize: 12.0)),
                        ),
                        // if (session.isClosed)
                        // Container(
                        //   margin: const EdgeInsets.only(left: 15.0, right: 20.0),
                        //   child: Text(lang.unfriended,
                        //     style: TextStyle(color: color.primary, fontSize: 12.0),
                        //   ),
                        // )
                    ]),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
