use fil_actors_shared::v13::EventBuilder;
use fvm_shared4::deal::DealID;
use fvm_shared4::ActorID;

trait WithParties {
    fn with_parties(self, id: DealID, client: ActorID, provider: ActorID) -> EventBuilder;
}

impl WithParties for EventBuilder {
    fn with_parties(self, id: DealID, client: ActorID, provider: ActorID) -> EventBuilder {
        self.field_indexed("id", &id)
            .field_indexed("client", &client)
            .field_indexed("provider", &provider)
    }
}
