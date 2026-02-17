//! Go Word - Desktop word processor application
//!
//! This is the main entry point for the Tauri desktop application.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod collab_commands;
mod commands;
mod mail_merge_commands;
mod revision_commands;
mod state;
mod template_commands;
mod view_mode_commands;

use commands::DocumentStore;
use state::{CollaborationState, FontManagerState, MailMergeState, PerfMetricsState, RevisionStateWrapper, SettingsState, TemplateState, ViewModeState};
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Go Word application");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize settings with app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            tracing::info!("App data directory: {:?}", app_data_dir);

            // Initialize settings state
            let settings_state = SettingsState::new(app_data_dir.clone());
            app.manage(settings_state);

            // Initialize font manager state
            tracing::info!("Initializing font manager...");
            let font_manager_state = FontManagerState::new();
            app.manage(font_manager_state);
            tracing::info!("Font manager initialized");

            // Initialize performance metrics state
            tracing::info!("Initializing performance metrics...");
            let perf_metrics_state = PerfMetricsState::new();
            app.manage(perf_metrics_state);
            tracing::info!("Performance metrics initialized");

            // Initialize template state
            tracing::info!("Initializing template manager...");
            let templates_dir = app_data_dir.join("templates");
            let template_state = TemplateState::new(templates_dir);
            app.manage(template_state);
            tracing::info!("Template manager initialized");

            // Initialize revision tracking state
            tracing::info!("Initializing revision tracking...");
            let revision_state = RevisionStateWrapper::new();
            app.manage(revision_state);
            tracing::info!("Revision tracking initialized");

            // Initialize view mode state
            tracing::info!("Initializing view mode state...");
            let view_mode_state = ViewModeState::new();
            app.manage(view_mode_state);
            tracing::info!("View mode state initialized");

            // Initialize collaboration state
            tracing::info!("Initializing collaboration state...");
            let collaboration_state = CollaborationState::new();
            app.manage(collaboration_state);
            tracing::info!("Collaboration state initialized");

            // Initialize mail merge state
            tracing::info!("Initializing mail merge state...");
            let mail_merge_state = MailMergeState::new();
            app.manage(mail_merge_state);
            tracing::info!("Mail merge state initialized");

            // Initialize document store
            tracing::info!("Initializing document store...");
            let doc_store = DocumentStore::default();
            app.manage(doc_store);
            tracing::info!("Document store initialized");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_document,
            commands::apply_command,
            commands::get_layout,
            commands::save_document,
            commands::load_document,
            commands::undo,
            commands::redo,
            commands::get_settings,
            commands::update_settings,
            commands::reset_settings,
            // Style system commands
            commands::get_styles,
            commands::get_style,
            commands::get_resolved_style,
            commands::apply_paragraph_style,
            commands::apply_character_style,
            commands::apply_direct_formatting,
            commands::clear_direct_formatting,
            commands::get_style_inspector,
            commands::create_style,
            commands::modify_style,
            // Font substitution commands
            commands::get_font_substitutions,
            commands::clear_font_substitutions,
            commands::get_available_fonts,
            commands::is_font_available,
            commands::resolve_font,
            // Bookmark commands
            commands::insert_bookmark,
            commands::delete_bookmark,
            commands::rename_bookmark,
            commands::go_to_bookmark,
            commands::list_bookmarks,
            commands::validate_bookmark_name,
            commands::bookmark_exists,
            // Autosave and recovery commands
            commands::get_recovery_files,
            commands::has_recovery_files,
            commands::recover_document,
            commands::discard_recovery,
            commands::discard_all_recovery,
            commands::get_autosave_status,
            commands::get_autosave_config,
            commands::set_autosave_config,
            // PDF export commands
            commands::export_pdf,
            commands::export_pdf_bytes,
            commands::get_pdf_export_options,
            // PDF/A export commands
            commands::export_pdf_a,
            commands::export_pdf_a_bytes,
            commands::validate_pdf_a_compliance,
            commands::get_pdfa_conformance_levels,
            // DOCX import/export commands
            commands::open_docx,
            commands::save_as_docx,
            commands::get_supported_formats,
            commands::get_import_formats,
            commands::get_export_formats,
            // Print commands
            commands::get_print_capabilities,
            commands::print_document,
            commands::render_preview_page,
            commands::render_preview_thumbnails,
            // Performance telemetry commands
            commands::get_performance_metrics,
            commands::reset_performance_metrics,
            commands::get_performance_budget,
            commands::set_performance_budget,
            commands::check_performance_budget,
            commands::set_performance_enabled,
            commands::is_performance_enabled,
            commands::record_command_timing,
            commands::record_layout_timing,
            commands::record_render_timing,
            commands::record_input_latency,
            // Template commands
            template_commands::list_templates,
            template_commands::get_template_metadata,
            template_commands::get_template_thumbnail,
            template_commands::create_from_template,
            template_commands::save_as_template,
            template_commands::delete_template,
            template_commands::search_templates,
            template_commands::filter_templates_by_category,
            template_commands::get_template_categories,
            template_commands::import_template,
            template_commands::export_template,
            // Style pack commands
            template_commands::export_style_pack,
            template_commands::import_style_pack,
            // Locked region commands
            template_commands::get_locked_regions,
            template_commands::set_locked_regions,
            template_commands::add_locked_region,
            template_commands::remove_locked_region,
            template_commands::clear_locked_regions,
            template_commands::is_position_locked,
            template_commands::validate_edit_for_locked_regions,
            // Comment commands
            commands::add_comment,
            commands::edit_comment,
            commands::delete_comment,
            commands::reply_to_comment,
            commands::edit_comment_reply,
            commands::delete_comment_reply,
            commands::resolve_comment,
            commands::reopen_comment,
            commands::get_comments,
            commands::get_comments_filtered,
            commands::get_comments_for_range,
            commands::get_comment,
            commands::get_comment_replies,
            commands::navigate_to_comment,
            commands::get_comment_count,
            // RTF import/export commands
            commands::import_rtf,
            commands::export_rtf,
            commands::import_rtf_bytes,
            commands::export_rtf_bytes,
            // ODT import commands (read-only)
            commands::import_odt,
            commands::import_odt_bytes,
            // Extended format support
            commands::get_all_import_formats,
            commands::get_all_export_formats,
            // Field commands
            commands::insert_field,
            commands::update_field,
            commands::update_all_fields,
            commands::toggle_field_codes,
            commands::lock_field,
            commands::get_field_result,
            commands::list_fields,
            commands::delete_field,
            commands::get_field_code,
            commands::evaluate_field,
            // Track Changes / Revision commands
            revision_commands::toggle_track_changes,
            revision_commands::enable_track_changes,
            revision_commands::disable_track_changes,
            revision_commands::is_tracking_changes,
            revision_commands::set_markup_mode,
            revision_commands::get_markup_mode,
            revision_commands::set_revision_author,
            revision_commands::get_revision_author,
            revision_commands::accept_revision,
            revision_commands::reject_revision,
            revision_commands::accept_all_revisions,
            revision_commands::reject_all_revisions,
            revision_commands::accept_revisions_by_author,
            revision_commands::reject_revisions_by_author,
            revision_commands::get_revisions,
            revision_commands::get_pending_revisions,
            revision_commands::get_revision,
            revision_commands::get_revisions_by_author,
            revision_commands::get_revision_authors,
            revision_commands::navigate_to_revision,
            revision_commands::get_next_revision,
            revision_commands::get_previous_revision,
            revision_commands::get_revision_summary,
            revision_commands::get_revision_state,
            revision_commands::set_revision_filter_authors,
            revision_commands::set_revision_filter_types,
            revision_commands::clear_revision_filters,
            revision_commands::clear_accepted_revisions,
            revision_commands::clear_rejected_revisions,
            revision_commands::set_author_color,
            revision_commands::get_author_color,
            // View mode commands
            view_mode_commands::get_view_mode,
            view_mode_commands::set_view_mode,
            view_mode_commands::get_view_mode_config,
            view_mode_commands::get_draft_options,
            view_mode_commands::set_draft_options,
            view_mode_commands::get_outline_options,
            view_mode_commands::set_outline_options,
            view_mode_commands::get_available_view_modes,
            view_mode_commands::get_draft_layout,
            view_mode_commands::promote_heading,
            view_mode_commands::demote_heading,
            view_mode_commands::move_section,
            view_mode_commands::expand_outline_heading,
            view_mode_commands::collapse_outline_heading,
            view_mode_commands::set_outline_level_filter,
            // Caption commands
            commands::insert_caption,
            commands::delete_caption,
            commands::edit_caption_text,
            commands::get_caption,
            commands::list_captions,
            commands::list_captions_by_label,
            commands::get_caption_format,
            commands::set_caption_format,
            commands::update_caption_numbers,
            commands::get_caption_for_target,
            commands::get_caption_labels,
            commands::add_custom_caption_label,
            commands::get_default_caption_formats,
            commands::get_caption_number_formats,
            // Pagination control commands
            commands::set_paragraph_keep_rules,
            commands::get_paragraph_keep_rules,
            commands::set_widow_orphan_control,
            commands::get_widow_orphan_control,
            commands::set_line_numbering,
            commands::get_line_numbering,
            commands::get_paragraph_pagination_info,
            // Cross-reference commands
            commands::insert_cross_reference,
            commands::delete_cross_reference,
            commands::update_cross_reference,
            commands::get_cross_reference,
            commands::list_cross_references,
            commands::get_available_targets,
            commands::navigate_to_target,
            commands::validate_cross_references,
            commands::update_all_cross_references,
            commands::get_broken_references,
            commands::get_display_options_for_type,
            commands::preview_cross_reference,
            commands::list_cross_references_by_type,
            commands::get_references_to_target,
            commands::get_cross_reference_statistics,
            // Text box commands
            commands::insert_text_box,
            commands::delete_text_box,
            commands::set_text_box_content,
            commands::set_text_box_style,
            commands::set_text_box_anchor,
            commands::resize_text_box,
            commands::get_text_box,
            commands::list_text_boxes,
            commands::enter_text_box_edit_mode,
            commands::exit_text_box_edit_mode,
            commands::get_text_box_edit_mode,
            // Shape CRUD commands
            commands::insert_shape,
            commands::delete_shape,
            commands::get_shape,
            commands::list_shapes,
            commands::update_shape_properties,
            // Shape text commands
            commands::set_shape_text,
            commands::get_shape_text,
            commands::set_text_auto_fit,
            // Shape styling commands
            commands::set_shape_fill,
            commands::set_shape_line,
            commands::set_shape_effects,
            // Shape group commands
            commands::group_shapes,
            commands::ungroup_shapes,
            commands::add_to_group,
            commands::remove_from_group,
            commands::get_shape_group,
            commands::list_shape_groups,
            // Shape alignment/distribution commands
            commands::align_shapes,
            commands::distribute_shapes,
            // Connector commands
            commands::insert_connector,
            commands::update_connector,
            commands::delete_connector,
            commands::get_connector,
            commands::list_connectors,
            commands::get_connectors_for_shape,
            // Z-order commands
            commands::bring_to_front,
            commands::send_to_back,
            commands::bring_forward,
            commands::send_backward,
            // Shape library commands
            commands::get_shape_types_by_category,
            commands::get_shape_categories,
            // Collaboration commands
            collab_commands::init_collaboration,
            collab_commands::close_collaboration,
            collab_commands::apply_crdt_op,
            collab_commands::apply_crdt_ops_batch,
            collab_commands::get_pending_ops,
            collab_commands::queue_local_op,
            collab_commands::ack_ops,
            collab_commands::get_sync_status,
            // Presence commands
            collab_commands::update_presence,
            collab_commands::remove_presence,
            collab_commands::get_remote_presence,
            collab_commands::get_active_users,
            collab_commands::assign_presence_color,
            // Version history commands
            collab_commands::get_version_history,
            collab_commands::create_named_version,
            collab_commands::create_checkpoint,
            collab_commands::restore_version,
            collab_commands::get_version,
            collab_commands::compare_versions,
            // Offline commands
            collab_commands::get_offline_status,
            collab_commands::set_connection_status,
            collab_commands::queue_offline_operation,
            collab_commands::get_offline_queue,
            collab_commands::clear_offline_queue,
            collab_commands::sync_complete,
            // Permission commands
            collab_commands::check_permission,
            collab_commands::grant_permission,
            collab_commands::revoke_permission,
            collab_commands::get_document_permissions,
            collab_commands::get_user_documents,
            collab_commands::create_share_link,
            collab_commands::redeem_share_link,
            // Mail merge commands
            mail_merge_commands::load_csv_data_source,
            mail_merge_commands::load_json_data_source,
            mail_merge_commands::load_csv_from_string,
            mail_merge_commands::load_json_from_string,
            mail_merge_commands::get_data_source_columns,
            mail_merge_commands::get_data_source_preview,
            mail_merge_commands::get_data_source_record,
            mail_merge_commands::get_data_source_records,
            mail_merge_commands::get_data_source_value,
            mail_merge_commands::list_data_sources,
            mail_merge_commands::remove_data_source,
            mail_merge_commands::clear_data_sources,
            mail_merge_commands::detect_csv_delimiter,
            mail_merge_commands::detect_csv_has_header,
            mail_merge_commands::get_data_source_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
