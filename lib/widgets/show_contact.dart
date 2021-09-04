import 'package:flutter/material.dart';

import 'package:provider/provider.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/apps/chat/models.dart' show Friend;
import 'package:esse/apps/chat/provider.dart';

class ContactList extends StatefulWidget {
  final Function callback;
  final bool multiple;
  const ContactList({Key? key, required this.callback, this.multiple = true})
      : super(key: key);

  @override
  _ContactListState createState() => _ContactListState();
}

class _ContactListState extends State<ContactList> {
  List<bool> _checks = [];
  Map<int, Friend> _friends = {};
  List<int> _keys = [];

  @override
  initState() {
    super.initState();
    new Future.delayed(Duration.zero, () {
      final provider = context.read<ChatProvider>();
      _friends = provider.friends;
      _keys = provider.orderKeys;
      _checks = List<bool>.generate(_keys.length, (_) => false);
      setState(() {});
    });
  }

  Widget _friend(int i, Friend friend) {
    return Container(
        height: 55.0,
        child: widget.multiple
            ? ListTile(
                leading: friend.showAvatar(),
                title: Text(friend.name),
                trailing: Checkbox(
                    value: _checks[i],
                    onChanged: (value) => setState(() => _checks[i] = value!)))
            : ListTile(
                onTap: () {
                  Navigator.pop(context);
                  widget.callback(friend.id);
                },
                leading: friend.showAvatar(),
                title: Text(friend.name),
              ));
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    double maxHeight = (MediaQuery.of(context).size.height - 400);
    if (maxHeight < 100.0) {
      maxHeight = 100.0;
    }

    return Column(children: [
      Container(
        height: 40.0,
        decoration: BoxDecoration(
            color: color.surface, borderRadius: BorderRadius.circular(15.0)),
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
            contentPadding:
                EdgeInsets.only(left: 15.0, right: 15.0, bottom: 15.0),
          ),
        ),
      ),
      const SizedBox(height: 16.0),
      Container(
          height: maxHeight,
          child: SingleChildScrollView(
              child: Column(
                  children: List<Widget>.generate(
                    _keys.length, (i) => _friend(i, _friends[_keys[i]]!))))),
      const SizedBox(height: 10.0),
      const Divider(height: 1.0, color: Color(0x40ADB0BB)),
      const SizedBox(height: 10.0),
      if (widget.multiple)
        Row(mainAxisAlignment: MainAxisAlignment.spaceEvenly, children: [
          TextButton(
            child: Text(lang.cancel, style: TextStyle(color: color.onSurface)),
            onPressed: () => Navigator.pop(context),
          ),
          TextButton(
            child: Text(lang.ok),
            onPressed: () {
              Navigator.pop(context);
              List<int> ids = [];
              _keys.asMap().forEach((i, value) {
                if (_checks[i]) {
                  ids.add(value);
                }
              });
              widget.callback(ids);
            },
          ),
        ])
    ]);
  }
}
