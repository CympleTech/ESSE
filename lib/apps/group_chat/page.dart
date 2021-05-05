import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/file_image.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/group_chat/add.dart';

class GroupChatList extends StatefulWidget {
  @override
  _GroupChatListState createState() => _GroupChatListState();
}

class _GroupChatListState extends State<GroupChatList> {
  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final isDesktop = isDisplayDesktop(context);

    return Scaffold(
      body: const Center(child: Text('TODO group list!')),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          final widget = GroupAddPage();
          if (isDesktop) {
            Provider.of<AccountProvider>(context, listen: false).updateActivedApp(widget);
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
