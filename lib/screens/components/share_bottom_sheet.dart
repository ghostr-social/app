import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:ghostr/src/rust/video/video.dart';
import 'package:share_plus/share_plus.dart';
import 'dart:io';
import 'package:share_plus/share_plus.dart';
import 'package:path/path.dart' as p;


/// A TikTok-style bottom sheet for sharing.
///
/// Usage:
/// ```dart
/// void _showShareBottomSheet(BuildContext context) {
///   showModalBottomSheet(
///     context: context,
///     isScrollControlled: true,
///     backgroundColor: Colors.transparent,
///     builder: (BuildContext context) {
///       return const ShareBottomSheet();
///     },
///   );
/// }
/// ```
class ShareBottomSheet extends StatelessWidget {
  final FfiVideoDownload video;
  const ShareBottomSheet({super.key,  required this.video});

  @override
  Widget build(BuildContext context) {
    // We use a DraggableScrollableSheet inside the bottom sheet so it can scroll
    // if the content overflows, or you can fix a certain height if you prefer.

    return Container(
      // Make the bottom sheet partially fill the screen (like ~ 60-70%)
      height: MediaQuery.of(context).size.height * 0.65,
      // Use a clip to get rounded corners at the top
      decoration: const BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.vertical(top: Radius.circular(16.0)),
      ),
      child: Column(
        children: [
          // Top row: search icon, title "Send to", close button
          _buildTopBar(context),
          // Expand the rest in a ScrollView so everything is scrollable
          Expanded(
            child: SingleChildScrollView(
              child: Padding(
                padding:
                const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
                child: Column(
                  children: [
                    // Suggested contacts (horizontal scroll)
                    _buildSuggestedContacts(),
                    const SizedBox(height: 10),
                    // Share grid
                    _buildShareGrid(),
                    const SizedBox(height: 10),
                    // Additional actions row
                    _buildAdditionalActions(),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  /// The top bar with search icon, title, and close button.
  Widget _buildTopBar(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 12.0, left: 16, right: 16),
      child: Row(
        children: [
          // Search icon
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () {
              // TODO: Implement actual search functionality if needed
            },
          ),
          Expanded(
            child: Text(
              "Send to",
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
          // Close button
          IconButton(
            icon: const Icon(Icons.close),
            onPressed: () {
              Navigator.of(context).pop(); // Dismiss bottom sheet
            },
          ),
        ],
      ),
    );
  }

  /// A horizontally scrollable row of suggested contacts or recent interactions.
  Widget _buildSuggestedContacts() {
    final suggestedContacts = [
      // You can populate this list from your backend or local data
      _ContactSuggestion(initial: 'C', name: 'nope.', color: Colors.purple),
      _ContactSuggestion(initial: 'J', name: 'Jane', color: Colors.blue),
      _ContactSuggestion(initial: 'M', name: 'Mike', color: Colors.red),
      _ContactSuggestion(initial: 'A', name: 'Alice', color: Colors.green),
      _ContactSuggestion(initial: 'D', name: 'David', color: Colors.orange),
    ];

    return SizedBox(
      height: 100, // Enough to show circular avatars + label
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        itemCount: suggestedContacts.length,
        separatorBuilder: (_, __) => const SizedBox(width: 12),
        itemBuilder: (context, index) {
          final item = suggestedContacts[index];
          return Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              CircleAvatar(
                radius: 30,
                backgroundColor: item.color,
                child: Text(
                  item.initial,
                  style: const TextStyle(color: Colors.white, fontSize: 25),
                ),
              ),
              const SizedBox(height: 6),
              Text(
                item.name,
                style: const TextStyle(fontSize: 14),
              )
            ],
          );
        },
      ),
    );
  }

  /// A grid of share options. Each option is an icon + label (e.g. WhatsApp).
  Widget _buildShareGrid() {
    // You can define your share options in a list of objects
    final shareOptions = [
      _ShareOption(
        icon:  FontAwesomeIcons.whatsapp,
        label: 'WhatsApp',
        onTap: () => _onShareTap('WhatsApp'),
      ),
      _ShareOption(
        icon: Icons.discord,
        label: 'Discord',
        onTap: () => _onShareTap('Discord'),
      ),
      _ShareOption(
        icon: Icons.message_outlined,
        label: 'SMS',
        onTap: () => _onShareTap('SMS'),
      ),
      _ShareOption(
        icon: Icons.link,
        label: 'Copy link',
        onTap: () => _onShareTap('Copy link'),
      ),
      _ShareOption(
        icon: Icons.repeat,
        label: 'Repost',
        onTap: () => _onShareTap('Repost'),
      ),
      _ShareOption(
        icon: Icons.more_horiz,
        label: 'More',
        onTap: () => _onShareTap('More'),
      ),
    ];

    // We want a grid with 4 columns
    return GridView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: shareOptions.length,
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 4, // 4 columns
        childAspectRatio: 0.75,
      ),
      itemBuilder: (context, index) {
        final option = shareOptions[index];
        return GestureDetector(
          onTap: option.onTap,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 50,
                height: 50,
                margin: const EdgeInsets.only(bottom: 5),
                decoration: BoxDecoration(
                  color: Colors.grey.shade200,
                  shape: BoxShape.circle,
                ),
                child: Icon(option.icon, size: 28, color: Colors.black87),
              ),
              Text(
                option.label,
                style:
                const TextStyle(fontSize: 12, fontWeight: FontWeight.w500),
                textAlign: TextAlign.center,
              ),
            ],
          ),
        );
      },
    );
  }

  /// Additional actions in a single row: Report, Not Interested, etc.
  Widget _buildAdditionalActions() {
    final additionalActions = [
      _ShareOption(
        icon: Icons.flag,
        label: 'Report',
        onTap: () => _onShareTap('Report'),
      ),
      _ShareOption(
        icon: Icons.visibility_off,
        label: 'Not Interested',
        onTap: () => _onShareTap('Not Interested'),
      ),
      _ShareOption(
        icon: Icons.local_fire_department,
        label: 'Promote',
        onTap: () => _onShareTap('Promote'),
      ),
      _ShareOption(
        icon: Icons.help_outline,
        label: 'Why this video?',
        onTap: () => _onShareTap('Why this video?'),
      ),
      _ShareOption(
        icon: Icons.closed_caption,
        label: 'Captions',
        onTap: () => _onShareTap('Captions'),
      ),
      _ShareOption(
        icon: Icons.control_camera,
        label: 'Duet',
        onTap: () => _onShareTap('Duet'),
      ),
    ];

    return SizedBox(
      height: 60,
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        itemCount: additionalActions.length,
        separatorBuilder: (_, __) => const SizedBox(width: 24),
        itemBuilder: (context, index) {
          final action = additionalActions[index];
          return GestureDetector(
            onTap: action.onTap,
            child: Column(
              children: [
                Icon(
                  action.icon,
                  size: 22,
                  color: Colors.black87,
                ),
                const SizedBox(height: 4),
                Text(
                  action.label,
                  style: const TextStyle(fontSize: 12),
                ),
              ],
            ),
          );
        },
      ),
    );
  }

  /// Called when a share option is tapped
  void _onShareTap(String optionName) async {
    final XFile file = XFile(video.localPath!);

    switch (optionName) {
      case 'WhatsApp':
        print("Trying to share to WhatsApp : ${video.localPath!}");
      // Share via system-level dialog, can also specify package name on Android
      // to attempt direct share to WhatsApp, but often the OS picks the best approach.
        await Share.shareXFiles(
          [file],
          text: 'Check out this video!',
          // If you specifically want to force WhatsApp on Android, you can attempt:
          // sharePositionOrigin: box!.localToGlobal(Offset.zero) & box.size, // For iPad
          // sharePackageName: 'com.whatsapp'
        );
        break;

      case 'Copy link':
      // Your original copy link handling
        Clipboard.setData(ClipboardData(text: "https://primal.net/e/${video.nostr.id}"));
        break;

      case 'SMS':
      // If you want to directly invoke an SMS intent, you could do something like:
      // url_launcher can help with "sms:xxx?body=Hey check this video"
      // Or just let the share sheet handle it:
        await Share.shareXFiles(
          [file],
          text: 'Check out this video!',
        );
        break;

      case 'More':
      // This generally opens the standard OS share sheet with all available apps
        await Share.shareXFiles(
          [file],
          text: 'Check out this video!',
        );
        break;

      default:
        debugPrint('Tapped $optionName but not specifically handled.');
        break;
    }
  }
}

/// Helper class for a contact suggestion to display in the horizontal list.
class _ContactSuggestion {
  final String initial;
  final String name;
  final Color color;

  _ContactSuggestion({
    required this.initial,
    required this.name,
    required this.color,
  });
}

/// Helper class that represents a share option with an icon, label, and onTap.
class _ShareOption {
  final IconData icon;
  final String label;
  final VoidCallback onTap;

  _ShareOption({
    required this.icon,
    required this.label,
    required this.onTap,
  });
}
