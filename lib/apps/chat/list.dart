import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';
import 'package:esse/session.dart';

import 'package:esse/apps/chat/provider.dart';
import 'package:esse/apps/chat/models.dart';
import 'package:esse/apps/chat/detail.dart';
import 'package:esse/apps/chat/add.dart';

class ChatList extends StatefulWidget {
  const ChatList({Key key}) : super(key: key);

  @override
  _ChatListState createState() => _ChatListState();
}

class _ChatListState extends State<ChatList> {
  @override
  Widget build(BuildContext context) {
    final provider = context.watch<ChatProvider>();
    final isDesktop = isDisplayDesktop(context);
    final friends = provider.friends;
    final chatKeys = provider.orderKeys;

    return Scaffold(
      body: ListView.builder(
        itemCount: chatKeys.length,
        itemBuilder: (BuildContext ctx, int index) => ListChat(friend: friends[chatKeys[index]]),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          final widget = ChatAddPage();
          if (isDesktop) {
            Provider.of<AccountProvider>(context, listen: false).updateActivedWidget(widget);
          } else {
            Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
          }
        },
        child: const Icon(Icons.add, color: Colors.white),
        backgroundColor: Color(0xFF6174FF),
      ),
    );
  }
}


class ListChat extends StatelessWidget {
  final Friend friend;
  const ListChat({Key key, this.friend}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () {
        context.read<AccountProvider>().updateActivedSession(0, SessionType.Chat, friend.id);
        context.read<ChatProvider>().updateActivedFriend(friend.id);
        if (!isDesktop) {
          Navigator.push(
            context,
            MaterialPageRoute(
              builder: (_) => ChatPage(),
            ),
          );
        } else {
          context.read<AccountProvider>().updateActivedWidget(ChatDetail());
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
              child: friend.showAvatar(),
            ),
            Expanded(
              child: Container(
                height: 55.0,
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Expanded(
                          child: Text(friend.name,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: const TextStyle(fontSize: 16.0)),
                        ),
                        if (this.friend.isClosed)
                        Container(
                          margin: const EdgeInsets.only(left: 15.0, right: 20.0),
                          child: Text(lang.unfriended,
                            style: TextStyle(color: color.primary, fontSize: 12.0),
                          ),
                        )
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
