permit(
  principal,
  action == Action::"GetBook",
  resource
) when {
  principal.is_admin || resource.user_id == principal.user_id
};