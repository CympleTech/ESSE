import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';
import 'package:esse/session.dart';
import 'package:esse/rpc.dart';

import 'package:esse/apps/group/detail.dart';
import 'package:esse/apps/group/models.dart';

class GroupChatList extends StatefulWidget {
  const GroupChatList({Key? key}) : super(key: key);

  @override
  _GroupChatListState createState() => _GroupChatListState();
}

class _GroupChatListState extends State<GroupChatList> {
  List<GroupChat> _groups = [];

  @override
  void initState() {
    super.initState();
    _loadGroups();
  }

  _loadGroups() async {
    this._groups.clear();
    final res = await httpPost('group-list', []);
    if (res.isOk) {
      res.params.forEach((params) {
          this._groups.add(GroupChat.fromList(params));
      });
      setState(() {});
    } else {
      print(res.error);
    }
  }

  @override
  Widget build(BuildContext context) {
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);

    return Scaffold(
      appBar: AppBar(title: Text(lang.groupChat)),
      body: Padding(
        padding: const EdgeInsets.symmetric(vertical: 10.0),
        child: ListView.builder(
          itemCount: _groups.length,
          itemBuilder: (BuildContext ctx, int index) => ListChat(group: _groups[index]),
        )
      ),
    );
  }
}

class ListChat extends StatelessWidget {
  final GroupChat group;
  const ListChat({Key? key, required this.group}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () {
        context.read<AccountProvider>().updateActivedSession(0, SessionType.Group, group.id);
        final widget = GroupChatDetail(id: group.id);
        if (!isDesktop) {
          Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
        } else {
          context.read<AccountProvider>().updateActivedWidget(widget);
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
              child: group.showAvatar(),
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
                          child: Text(group.name,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(fontSize: 16.0))
                        ),
                        if (group.isClosed)
                        Container(
                          margin: const EdgeInsets.only(left: 15.0, right: 20.0),
                          child: Text(lang.closed,
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
