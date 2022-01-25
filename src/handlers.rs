async fn default(a11y: &Accessible) -> Result<dbus::Error, String> {
  return a11y.get_text().await?;
}

async fn heading(a11y: &Accessible) -> Result<dbus::Error, String> {

}

async fn to_text(a11y: &Accessible) {
  return match (a11y.get
}
