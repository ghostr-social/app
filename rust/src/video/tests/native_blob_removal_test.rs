use super::support::temp_directory;
use crate::video::native_blob_integrity::remove_if_present;

#[tokio::test]
async fn native_blob_removal_handles_files_directories_and_absence() {
    let root = temp_directory("ghostr-blob-removal");
    let file = root.join("clip.mp4");
    tokio::fs::write(&file, b"video").await.expect("write file");
    remove_if_present(&file).await.expect("remove file");
    assert!(!file.exists());

    let directory = root.join("clip");
    tokio::fs::create_dir(&directory)
        .await
        .expect("create directory");
    tokio::fs::write(directory.join("nested"), b"video")
        .await
        .expect("write nested file");
    remove_if_present(&directory)
        .await
        .expect("remove directory");
    assert!(!directory.exists());
    remove_if_present(&root.join("missing"))
        .await
        .expect("ignore missing");

    let plain = root.join("plain");
    tokio::fs::write(&plain, []).await.expect("plain file");
    let error = remove_if_present(&plain.join("child"))
        .await
        .expect_err("invalid parent must fail");
    assert!(error.to_string().contains("inspect native blob"));
    std::fs::remove_dir_all(root).expect("remove test directory");
}
