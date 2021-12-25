import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/utils/better_print.dart';
import 'package:esse/utils/pick_image.dart';
import 'package:esse/utils/pick_file.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/widgets/emoji.dart';
import 'package:esse/widgets/button_text.dart';
import 'package:esse/widgets/input_text.dart';
import 'package:esse/widgets/shadow_dialog.dart';
import 'package:esse/widgets/audio_recorder.dart';
import 'package:esse/widgets/show_contact.dart';
import 'package:esse/global.dart';
import 'package:esse/options.dart';
import 'package:esse/rpc.dart';

class DaoDetail extends StatefulWidget {
  const DaoDetail({Key? key}) : super(key: key);

  @override
  _DaoDetailState createState() => _DaoDetailState();
}

class _DaoDetailState extends State<DaoDetail> {
  final GlobalKey<ScaffoldState> _scaffoldKey = GlobalKey<ScaffoldState>();

  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    final reallyDesktop = isDisplayDesktop(context);
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final width = MediaQuery.of(context).size.width;

    bool isDesktop = true;
    if (width - 520 < 500) {
      isDesktop = false;
    }

    return Scaffold(
      key: _scaffoldKey,
      drawer: _ChannelScreen(),
      endDrawer: _MemberScreen(),
      drawerScrimColor: const Color(0x26ADB0BB),
      drawerEnableOpenDragGesture: false,
      appBar: AppBar(
        automaticallyImplyLeading: false,
        leading: reallyDesktop ? null : IconButton(icon: Icon(Icons.arrow_back),
          onPressed: () => Navigator.pop(context)),
        title: Text('ESSE', maxLines: 1, overflow: TextOverflow.ellipsis),
        bottom: isDesktop ? PreferredSize(
          child: Container(color: const Color(0x40ADB0BB), height: 1.0),
          preferredSize: Size.fromHeight(1.0)): null,
        actions: [
          if (!isDesktop)
          IconButton(icon: Text('#',
              style: TextStyle(color: color.primary, fontSize: 20.0, fontWeight: FontWeight.bold)),
            onPressed: () => _scaffoldKey.currentState!.openDrawer(),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20.0),
            child: IconButton(
              icon: Icon(Icons.people),
              onPressed: () => _scaffoldKey.currentState!.openEndDrawer(),
            ),
          )
        ]
      ),
      body: Container(
        alignment: Alignment.topCenter,
        child: isDesktop
        ? Row(children: [
            Expanded(child: _MainScreen()),
            _ChannelScreen(),
        ])
        : _MainScreen()
      )
    );
  }
}

class _MainScreen extends StatefulWidget {
  _MainScreen({Key? key}) : super(key: key);

  @override
  _MainScreenState createState() => _MainScreenState();
}

class _MainScreenState extends State<_MainScreen> {
  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Column(
      children: [
        Expanded(
          child: ListView.builder(
            padding: EdgeInsets.symmetric(horizontal: 20.0),
            itemCount: 0,
            reverse: true,
            itemBuilder: (BuildContext context, index) {
              return Container();
            }
        )),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 10.0),
          child: Row(
            children: [
              GestureDetector(
                onTap: () {},
                child: Container(width: 20.0,
                  child: Icon(Icons.mic_rounded, color: color.primary)),
              ),
              const SizedBox(width: 10.0),
              Expanded(
                child: Container(
                  height: 40,
                  decoration: BoxDecoration(
                    color: color.surface,
                    borderRadius: BorderRadius.circular(10.0),
                  ),
                  child: TextField(
                    //enabled: isOnline,
                    style: TextStyle(fontSize: 14.0),
                    textInputAction: TextInputAction.send,
                    onChanged: (value) {
                      //
                    },
                    //onSubmitted: (_v) => _sendMessage(),
                    decoration: InputDecoration(
                      hintText: 'Aa',
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.only(
                        left: 15.0, right: 15.0, bottom: 7.0),
                    ),
                    //controller: textController,
                    //focusNode: textFocus,
                  ),
                ),
              ),
              SizedBox(width: 10.0),
              GestureDetector(
                onTap: () {},
                child: Container(width: 20.0,
                  child: Icon(Icons.emoji_emotions_rounded, color: color.primary)),
              ),
              SizedBox(width: 10.0),
              GestureDetector(
                onTap: () {},
                child: Container(width: 20.0,
                  child: Icon(Icons.add_circle_rounded, color: color.primary)),
              ),
            ],
          ),
        ),
      ]
    );
  }
}

class _ChannelScreen extends StatefulWidget {
  _ChannelScreen({Key? key}) : super(key: key);

  @override
  _ChannelScreenState createState() => _ChannelScreenState();
}

class Channel {
  String name = '';
  bool public = true;
  bool selected = false;

  Channel(this.name, [this.public=true]);
}

class Category {
  List<Channel> channels = [];
  String name = '';
  bool open = true;

  Category(this.channels, this.name);
}

class _ChannelScreenState extends State<_ChannelScreen> {
  List<Category> _categories = [
    Category([Channel('general'), Channel('esse'), Channel('privary', false)], 'common'),
    Category([Channel('english'), Channel('中文', false)], 'language'),
    Category([Channel('game'), Channel('social')], 'voice'),
  ];

  @override
  void initState() {
    super.initState();
  }

  Widget _channel(Channel channel, color) {
    return Container(
      margin: const EdgeInsets.symmetric(vertical: 2.0, horizontal: 10.0),
      decoration: channel.selected ? BoxDecoration(
        borderRadius: BorderRadius.circular(5.0),
        color: Color(0x40ADB0BB)
      ) : BoxDecoration(borderRadius: BorderRadius.circular(5.0)),
      child: TextButton(
        onPressed: () => setState(() {
            this._categories.forEach((category) {
                category.channels.forEach((channel) {
                    channel.selected = false;
                });
            });
            channel.selected = true;
        }),
        child: Row(
          children: [
            SizedBox(
              width: 30.0,
              height: 30.0,
              child: Center(
                child: channel.public
                ? Text('#', style: TextStyle(
                    fontSize: 20.0,
                    color: channel.selected ? color.primary : Colors.grey
                ))
                : Icon(Icons.lock_rounded, size: 18.0,
                  color: channel.selected ? color.primary : Colors.grey
                ),
            )),
            Expanded(child: Text(channel.name.toLowerCase(), style: TextStyle(
                  color: channel.selected ? color.primary : Colors.grey
            ))),
          ]
        ),
      )
    );
  }

  Widget _category(Category category, color) {
    return Container(
      padding: const EdgeInsets.only(right: 10.0, top: 10.0),
      child: Row(
        children: [
          TextButton(
            style: TextButton.styleFrom(primary: color.onSurface.withOpacity(0.75)),
            onPressed: () => setState(() =>
              category.open = !category.open,
            ),
            child: Row(
              children: [
                Icon(category.open ? Icons.expand_more : Icons.expand_less, size: 16.0),
                SizedBox(
                  width: 110.0,
                  child: Text(
                    category.name.toUpperCase(), maxLines: 1, overflow: TextOverflow.ellipsis,
                    textAlign: TextAlign.left,
                  )
                )
              ]
          )),
          Spacer(),
          SizedBox(
            width: 40.0,
            child: TextButton(
              child: Icon(Icons.add, size: 18.0),
              onPressed: () {}
          ))
    ]));
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return SafeArea(
      child: Container(
        width: 200.0,
        decoration: BoxDecoration(color: color.secondary),
        child: ListView(
          children: _categories.map((category) =>
            Column(
              children: [
                _category(category, color),
                category.open ? Column(
                  children: category.channels.map((channel) =>
                    _channel(channel, color)).toList()
                ) : Container(),
              ]
            )
          ).toList()
        )
    ));
  }
}

class _MemberScreen extends StatefulWidget {
  _MemberScreen({Key? key}) : super(key: key);

  @override
  _MemberScreenState createState() => _MemberScreenState();
}

class _MemberScreenState extends State<_MemberScreen> {
  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return SafeArea(
      child: Container(
        width: 200.0,
        decoration: BoxDecoration(color: color.secondary),
        child: ListView(
          children: [
            //
            ListTile(title: Text('Sun')),
            ListTile(title: Text('Sun')),
            ListTile(title: Text('Sun')),
            ListTile(title: Text('Sun')),
          ]
        )
    ));
  }
}
