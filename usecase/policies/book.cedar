permit(
  principal,
  action in [
    Action::"ListBooks",
    Action::"GetBook",
    Action::"CreateBook",
    Action::"UpdateBook",
    Action::"DeleteBook",
    Action::"ChangeBookAppliedAt"
  ],
  resource
) when {
  principal.is_admin || resource.user_id == principal.user_id
};

permit(
  principal,
  action in [
    Action::"ListPublishers",
    Action::"GetPublisher",
    Action::"CreatePublisher",
    Action::"UpdatePublisher",
    Action::"DeletePublisher"
  ],
  resource
);

permit(
  principal,
  action in [
    Action::"ListShops",
    Action::"GetShop",
    Action::"CreateShop",
    Action::"UpdateShop",
    Action::"DeleteShop"
  ],
  resource
);

permit(
  principal,
  action == Action::"GetAnnualSummary",
  resource
);
