//! Builder functions for dashboard feature detail views.

use chrono::Utc;

use crate::templates::{
    EvidenceBundleView, FeatureView, MediaAssetView, ReportArtifactView, WpView,
};

/// Build evidence bundles for a feature and its work packages.
pub(super) fn build_feature_evidence_bundles(
    feature: &FeatureView,
    workpackages: &[WpView],
) -> Vec<EvidenceBundleView> {
    let mut bundles = vec![EvidenceBundleView {
        id: format!("bundle-{id}-summary", id = feature.id),
        fr_id: format!("FR-{id}", id = feature.id),
        evidence_type: "feature_summary".into(),
        wp_id: "dashboard".into(),
        wp_title: feature.title.clone(),
        artifact_path: format!("/artifacts/features/{}.md", feature.slug),
        created_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        artifact_ext: "md".into(),
        status: "available".into(),
        content_preview: Some("# Feature Summary\n\nThis feature provides...".to_string()),
        is_text_artifact: true,
        is_image_artifact: false,
        download_url: format!("/api/evidence/{}/summary/content", feature.id),
        test_passed: None,
        tests_passed_count: 0,
        tests_failed_count: 0,
        test_summary: None,
        commit_count: 0,
        pr_count: 0,
        ci_links: vec![],
        git_commits: vec![],
        pr_links: vec![],
    }];

    for wp in workpackages {
        bundles.push(EvidenceBundleView {
            id: format!("bundle-{fid}-wp-{wid}", fid = feature.id, wid = wp.id),
            fr_id: format!("FR-{fid}", fid = feature.id),
            evidence_type: "workpackage_artifact".into(),
            wp_id: wp.id.to_string(),
            wp_title: wp.title.clone(),
            artifact_path: format!(
                "/artifacts/wp/{wid}/{slug}.json",
                wid = wp.id,
                slug = feature.slug
            ),
            created_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            artifact_ext: "json".into(),
            status: if wp.progress > 0 {
                "accepted"
            } else {
                "generated"
            }
            .into(),
            content_preview: Some(r#"{"status":"generated","progress":0}"#.to_string()),
            is_text_artifact: true,
            is_image_artifact: false,
            download_url: format!("/api/evidence/{}/{}/content", feature.id, wp.id),
            test_passed: None,
            tests_passed_count: 0,
            tests_failed_count: 0,
            test_summary: None,
            commit_count: 0,
            pr_count: 0,
            ci_links: vec![],
            git_commits: vec![],
            pr_links: vec![],
        });
    }

    bundles
}

/// Build media assets for a feature and its work packages.
pub(super) fn build_feature_media_assets(
    feature: &FeatureView,
    workpackages: &[WpView],
) -> Vec<MediaAssetView> {
    let mut media = vec![MediaAssetView {
        id: format!("media-{id}-cover", id = feature.id),
        source: "dashboard".into(),
        name: format!("{slug}-hero.png", slug = feature.slug),
        kind: "image".into(),
        mime: "image/png".into(),
        url_or_path: format!("/assets/{slug}/cover.png", slug = feature.slug),
        size_bytes: 128_512,
        uploaded_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    }];

    for wp in workpackages {
        media.push(MediaAssetView {
            id: format!("media-{fid}-wp-{wid}", fid = feature.id, wid = wp.id),
            source: "agent-work-package".into(),
            name: format!("{slug}-wp-{wid}.png", slug = feature.slug, wid = wp.id),
            kind: "screenshot".into(),
            mime: "image/png".into(),
            url_or_path: format!("/assets/wp/{wid}/coverage.png", wid = wp.id),
            size_bytes: 84_320 + (wp.id as usize * 3_000),
            uploaded_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        });
    }

    media
}

/// Build report artifacts for a feature.
pub(super) fn build_feature_reports(
    feature: &FeatureView,
    workpackages: &[WpView],
) -> Vec<ReportArtifactView> {
    vec![ReportArtifactView {
        id: format!("report-{id}-coverage", id = feature.id),
        name: format!("Feature Coverage Report — {name}", name = feature.title),
        source: "coverage-engine".into(),
        status: "completed".into(),
        generated_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        rule_count: 5,
        satisfied_count: if feature.labels.is_empty() {
            2
        } else {
            feature.labels.len() + 2
        },
        compliant: !workpackages.is_empty(),
    }]
}
